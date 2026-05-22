---
doc_class: User-Journey-Story
journey_id: j158-print-shop-cell-rebalance-shorts-creator-spike
date: 2026-05-20
authority_tier: 2
status: draft
---

# j158 — Story: 14:18 KST Wednesday in Mapo-gu, the cell is groaning

## §0 — Wednesday March 17, 2027, 14:18 KST

Spring in Seoul. The cherry blossoms along the Han River have started to bud but it's still cool — 12°C in Mapo-gu under thin overcast. The Sungkyul-Sangsa Print Shop occupies the third floor of a five-story commercial building on Wausan-ro near Hongdae station, a 600-meter walk from Hae-Won Kim's one-room apartment.

Hae-Won (김해원, hangul: 김해원; family name first per Korean convention; 29 years old) is at her desk at the mailroom — a glass-walled corner of the third floor where she coordinates inter-departmental logistics: incoming customer orders, courier dispatch (CJ Logistics + Hanjin), inter-cell shuttling between the company's two Seoul plants, and the steady stream of small print jobs that flow through Sungkyul-Sangsa's day-to-day operation. Her desk has a single 27" monitor, a Wacom Cintiq Pro 24 she bought herself (for personal-time content work), her Samsung Galaxy S25 Ultra clipped to a desk-mount, and a Daiso ceramic mug with Earl Grey going cold.

It is **lunch shift end**. She just came back from a tteokbokki place around the corner with her colleague Park Jaewon (the binding-line lead). Her phone is on silent because Lee Min-Jun (사장님, the owner) frowns on phones at desks even though Hae-Won's own oyatie tablet is always there for legitimate work.

At **14:18:22 KST** the oyatie tablet chirps. It's the active-tenant chip flipping: it had been showing `Sungkyul-Sangsa · 마포-1` (employer tenant, work tenant) all morning. It now flickers, briefly, to show a small notification badge on the personal-tenant chip layered behind it.

She taps. The notification is from her **personal tenant** `personal-haewon-kim-kr`. It's the autoscale signal:

> 📈 **@haewon_paperlife** autoscale active
> short "8시간 동안 종이 접는 소리만" → 21.7M views
> cell `kr-seoul-shorts-creator-tier-4` at 8.4× baseline
> autoscale event-id `as-2027-03-17-1418-haewon-shorts-007`

She breathes out. She has been watching this short since Monday. On Monday afternoon it crossed 1M. Tuesday morning, 8M. Tuesday night, 14M. Wednesday lunch, 19M. Now 21.7M. The watch-time + retention curves are absurd — average watch-through is 94%, which for a 28-second short means people are watching the full thing more than nine out of ten times.

The autoscale signal is internal to her personal tenant. Nothing crosses to her employer side automatically. Her oyatie phone shows the active-tenant pill very clearly: this notification is in `personal-haewon-kim-kr` only. The boundary holds.

But Hae-Won knows what is about to happen because she has been watching the inbound emails to Sungkyul-Sangsa's `info@` address: Monday afternoon a handful of new customer inquiries; Tuesday a steady trickle; Wednesday morning a flood. She has been quietly funneling them to the right departments. Lee Min-Jun has been distracted in board meetings. He has not yet connected the dots.

She decides to make the connection explicit.

## §1 — 14:24 KST: the disclosure signal

She opens the messenger app. The thread she wants doesn't exist yet — she has to invoke the **creator-employer disclosure signal** pathway. It's a specific Cedar-gated path that ADR-0311 introduced when oyatie shipped the cross-tenant disclosure-permit family. Hae-Won signed her side-business disclosure with Lee Min-Jun back on **2024-08-12** when she started her shorts hobby seriously. That disclosure record is what makes this signal possible.

She taps "creator-employer disclosure signal" in the menu. A modal opens:

```
─── DISCLOSURE SIGNAL — personal → employer ───

To:    Sungkyul-Sangsa Print Co. (your employer)
       Routed to: Lee Min-Jun (사장님) + your own employer-tenant inbox

This is a ONE-WAY INFO-ONLY signal. The signal:
  ✓ contains no audience PII
  ✓ contains no revenue figures
  ✓ contains no audience demographics
  ✗ payload size ≤ 1024 bytes
  ✗ cannot be used to query your personal tenant

You may include:
  • a coarse-grained "spike happening" assertion
  • a heads-up timeline (when, where, expected duration)
  • an offer to help (optional)

Disclosure record: disclosure-haewon-kim-sungkyul-sangsa-2024-08-12 ✓ active

───────────────────────────────────────────────
```

She writes (in Korean):

> 사장님 안녕하세요. 저의 개인 채널 (@haewon_paperlife) 에 올린 종이접기 ASMR 영상이 갑자기 바이럴 됐어요 — 지금까지 21.7M views, 한국 #2 trending. Mon-Wed 사이에 우리 회사로 문의 메일이 평소보다 3-4배 정도 들어오고 있다고 생각해요. 만약 도움이 필요하시면 제가 셀 재조정 워크플로우를 시작할 수 있어요. 알려주세요.
>
> (English gloss: "Sajangnim hello. My personal channel @haewon_paperlife's paper-folding ASMR short went viral — 21.7M views, trending #2 in Korea. I think inquiries to our company are running 3-4× normal Mon-Wed. If you need help, I can start the cell-rebalance workflow. Let me know.")

She taps send. The Cedar evaluation runs in 89 ms:

- Principal: `haewon.kim@personal-haewon-kim-kr`
- Action: `messenger.creator_employer_disclosure_signal`
- Resource: `Tenant::"sungkyul-sangsa-print-co-kr"`
- Context: `payload_class == "creator_spike_info_only"`, `payload_no_audience_pii == true`, `payload_no_revenue_figures == true`, `payload_max_size_bytes == 612`, `disclosure_active == true`

Permit. The message lands in Lee Min-Jun's employer-tenant messenger inbox at 14:24:18 KST. Simultaneously, Hae-Won's own employer-tenant principal (her work-role principal `haewon.kim@sungkyul-sangsa-print-co-kr` for logistics coordinator) receives a copy.

`EVT-J158-DISCLOSURE-SIGNAL-002` dual-sealed in BOTH tenants. Specifically: in `personal-haewon-kim-kr` (source) and `sungkyul-sangsa-print-co-kr` (destination). Critically, no other data crossed. No audience metrics. No revenue. No DM threads.

## §2 — 14:31 KST: Lee Min-Jun reads + responds

Lee Min-Jun is 58, the second-generation owner. He inherited Sungkyul-Sangsa from his father in 2009 after a 12-year career in pharmaceutical marketing. He runs the print shop with a careful traditional hand but is more tech-curious than his peer cohort. He uses the oyatie ops console on a dual-monitor setup in his office at the back of the third floor.

At 14:31:14 KST he reads the disclosure signal. He smiles slightly. He has been watching the inquiry inbox tick up for two days. He had not connected it to Hae-Won — she has always been careful not to mention her creator life at work and he respects her boundary. The disclosure signal is the bridge she chose to build, on her terms.

He types in Korean:

> 김해원 님 감사해요. 도움 부탁드릴게요. 지금 셀 재조정 워크플로우 시작해주세요. 저는 박재원이랑 같이 inquiry 응답 분담 정리할게요. 시급은 추가 시간 정상 보전.
>
> (English gloss: "Hae-Won-nim, thank you. Please help. Start the cell-rebalance workflow now. I'll work with Park Jae-Won to organize inquiry response delegation. Your hours will be properly compensated.")

He adds a 👍 reaction.

The exchange happened entirely within `sungkyul-sangsa-print-co-kr`. The personal-tenant side of Hae-Won never saw Lee's response — it lives in her employer-tenant inbox. The boundary holds inverse: employer → personal carries no data path. Cedar's FORBID-1 (employer query against personal) is the immutable side of this story.

## §3 — 14:34–14:58 KST: cell-rebalance workflow initialization

Hae-Won switches active-tenant from personal to employer. The pill flips: `Sungkyul-Sangsa · 마포-1 (logistics coordinator)`. She opens her work-side `tasks` app. There is already a high-priority work item materialized:

```
[TASK] cell-rebalance workflow proposed
state: capacity_signal_detected
auto-generated from: disclosure-signal-haewon-2027-03-17-1424
expected order-intake spike: 3.7× over Mon-Wed baseline
cells in scope: 4 (1 primary + 2 burst + 1 secondary)
authority required: logistics-coordinator + owner-co-sign
```

She taps to open. The workflow-engine has materialized a 5-state rebalance lifecycle:

1. `capacity_signal_detected` ← we are here
2. `rebalance_proposed`
3. `cross_cell_grant_negotiated`
4. `traffic_shift`
5. `post_rebalance_validation`

At 14:38:12 KST she advances to `rebalance_proposed`. The workflow-engine generates a proposal:

- Bring online `kr-seoul-employer-print-shop-burst-1` (warm-spare cell; cold-start to ready in ~22 min)
- Bring online `kr-seoul-employer-print-shop-burst-2` (warm-spare cell; cold-start ~22 min)
- Allocate cross-cell capacity grants from the company's reserved capacity pool (Korean-region reserved capacity is 4 cell-equivalent units; this proposal uses 2 of them)
- Estimated burst window: 4 days (Wed → Sun); will reassess at 04:00 KST daily

Lee Min-Jun receives the proposal in his own ops console at 14:38:42 KST. He signs at 14:42:08 (passkey + face_id). The workflow advances to `cross_cell_grant_negotiated` at 14:42:14 KST.

The `cell` µservice initiates the cold-start of both burst cells. `kr-seoul-employer-print-shop-burst-1` reaches `ready` at 14:58:11. `burst-2` reaches `ready` at 15:01:42.

`EVT-J158-CELLS-WARMED-003` sealed in `sungkyul-sangsa-print-co-kr`.

## §4 — 14:58–16:32 KST: traffic shift

Hae-Won initiates the traffic shift at 15:02:18 KST. The plan:

- Move 32% of incoming order intake to `burst-1`
- Move 28% to `burst-2`
- Reserve 40% on `primary` for steady-state workflows
- Failover to `secondary` if any of the above degrade
- Production-planning gets 3.7× capacity allocation for the next 96 hours

The traffic shift is a gradual ramp — over 90 minutes, traffic moves from 100/0/0 (primary/burst-1/burst-2) to 40/32/28. The `workflow-engine` orchestrates the percentage shift in 10-minute increments. Each 10-min increment Cedar-evaluates and audits.

By 16:32 KST the traffic distribution stabilizes at the target ratio. Latency p95 on each cell:

- primary: 142 ms (was 189 ms before rebalance)
- burst-1: 168 ms (cold-start; will warm down over next 30 min)
- burst-2: 173 ms

`EVT-J158-REBALANCE-TRAFFIC-SHIFT-004` sealed throughout.

In parallel, `tasks` materializes 18 new order intakes from the morning queue (these are NEW SMB customers who saw Hae-Won's short and tracked down Sungkyul-Sangsa as the shop in the video). Each new inquiry gets a CRM record, an SLA timer (4-hour response target), and a routing to either Park Jae-Won (binding-focused) or Hae-Won (logistics + multi-product).

## §5 — 16:32–17:14 KST: production planning re-plan

The `production-planning` µservice ingests the rebalance + the new order intakes. It computes:

- Additional press shifts needed: 1 evening shift (16:00–22:00 KST) + 1 night shift (22:00–04:00 KST) over the next 4 days
- Paper inventory: current stock 14,200 sheets Munken 70gsm + 8,400 sheets coated 100gsm; need to order +20,000 sheets coated 90gsm by Friday for the new orders
- Binding capacity: book-binding line 2 needs to come online; Park Jae-Won assigned

The KR-LSA evaluator runs at every step:

- Hae-Won's projected weekly hours after burst: 38.5 (cap 52 with overtime; well under)
- Park Jae-Won's projected weekly hours: 47.2 (cap 52; within range but flagged for monitoring)
- Lee Min-Jun's projected weekly hours: 51.8 (cap 52; tight; Hae-Won surfaces a recommendation to redistribute)

`EVT-J158-PRODUCTION-REPLAN-005` sealed.

`EVT-J158-KR-LSA-EVALUATION-005a` sealed: green for Hae-Won, green for Park (monitor), yellow for Lee.

## §6 — 17:14–18:42 KST: post-rebalance validation + post-mortem prep

At 17:14 KST Hae-Won advances the workflow to `post_rebalance_validation`. The state-machine guard requires:

- Latency p95 on all active cells ≤ 200 ms ✓
- Order-intake successfully routed across cells ✓ (no orphaned messages in the queue)
- KR-LSA evaluator green ✓ (with the Lee Min-Jun yellow flag noted)
- All audit-chain seals coherent across the two tenants ✓
- Dual-tenant boundary invariant test passes ✓

She runs the boundary invariant probe explicitly: from the employer-tenant ops console, attempt to query `personal-haewon-kim-kr`. Cedar evaluates: `forbid`. The probe is denied with audit `EVT-J158-CEDAR-DENY-EMPLOYER-TO-PERSONAL-008` dual-sealed. The dual-seal validates the dual-tenant boundary — the deny is recorded in BOTH tenants, even though the probe came from the employer side.

The reverse probe — Hae-Won (personal) attempting to read employer tenant data without an active permit — also denies. Cedar's forbid rule for the reverse direction is symmetrical, and the dual-tenant boundary holds.

`EVT-J158-POST-REBALANCE-VALIDATION-009` sealed at 18:42:18 KST.

## §7 — 18:42 KST: end-of-day + the long view

Hae-Won finishes the work-side tasks for the day. Lee Min-Jun pops his head into the mailroom around 18:48 to thank her. He doesn't mention the shorts. He simply says "고생 많았어요" (you worked hard).

She walks home through the Hongdae backstreets. It is dark by 18:54 KST in mid-March. She stops at the GS25 convenience store on her corner for an evening shrimp triangular kimbap and a Cass beer. At 19:08 in her apartment she opens her personal tenant on her phone.

The short is at 23.1M views. The DMs are full. There are 4,200 new follow requests on her creator side. None of this crosses to her employer tenant. The boundary holds.

She types a short reply to one of her creator DMs — a fan asking what kind of paper makes the best sound. She writes "Munken 100gsm — the soft fibers". She tags it as a future short idea in her personal-tenant notes.

Tomorrow morning at 09:00 KST the burst cells will continue running. The order intake will normalize over Thu-Fri-Sat-Sun. By Monday, traffic will likely return to 100% primary and the burst cells will gracefully decommission. The autoscale event will fade. Hae-Won will be in the mailroom doing what she has always done.

The dual-tenant boundary makes it possible. The disclosure permit gave her agency over what her employer learned. The cellular architecture absorbed the spike without spilling state. None of this is novel as architectural ideas — but the substrate that allowed all three to coordinate cleanly is the thing oyatie is trying to be.

## §8 — Beats not on the wire (the human texture)

- At 14:24 KST, Hae-Won held the "send" button on the disclosure signal for about 12 seconds before she actually tapped. She was nervous. The disclosure record from 2024-08-12 was about "if you do shorts of the print shop, please tell me beforehand so I can sign off on what you film" — Lee Min-Jun's hand-written addendum in the disclosure form. She had not asked herself, before today, whether the disclosure went both ways.
- At 14:31 KST, Lee Min-Jun walked from his office to the binding line to find Park Jae-Won. They talked for 8 minutes. Park's first question was "the kid in the video is Hae-Won? I had no idea." Lee said "she's careful about it. don't tease her."
- At 15:42 KST Hae-Won received a DM on her personal-tenant creator inbox from her mother in Daegu: "엄마가 너의 영상 보고있어. 너무 자랑스러워." ("Mom is watching your video. So proud.") She read it during a coffee break. It is the kind of message that does not appear in any audit log but matters more than any of them.
- At 18:42 KST when the post-rebalance validation sealed, Hae-Won noticed her own work-tenant cell-rebalance dashboard showed "validated by haewon.kim@sungkyul-sangsa-print-co-kr (logistics coordinator)" — the role name her grandfather, also a print-shop worker in 1970s Busan, never had a title for. She thought about him for a second. Then she closed the tab.

## §9 — Stop condition for this story

This story documents the lived texture of the 4h24m journey from the autoscale signal to the post-rebalance validation. The acceptance criteria in `README.md`, the API shapes in `handshake.md`, the test cases in `integration-test-plan.md`, and the schema files together encode the machine semantics. The story exists so the next reader understands WHY the creator-employer disclosure permit is one-way + info-only, WHY the dual-tenant boundary inverts even when the same human is on both sides, and WHY the cell-rebalance is a local-tenant operational concern rather than a cross-tenant cascade.
