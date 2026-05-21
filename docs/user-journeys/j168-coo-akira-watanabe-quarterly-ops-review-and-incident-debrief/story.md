---
doc_class: User-Journey-Story
journey_id: j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief
date: 2026-05-20
authority_tier: 2
status: draft
---

# j168 — Story: 06:42 JST in Tamachi, the quarterly review opens

## §0 — Sunday May 10, 2026, 22:18 JST — Haneda Terminal 3, arrival

Akira Watanabe disembarks from ANA flight NH173 (Houston IAH → Tokyo Haneda HND, scheduled 13:55 CDT departure, 18:25 JST arrival; actual landing 22:08 JST after a 27-minute holding pattern over Boso Peninsula due to a typhoon-margin headwind from a remnant of Typhoon-19 east of Honshu). She is 51, wearing a charcoal A.P.C. crewneck sweater + dark indigo Japanese-selvedge jeans + black Onitsuka Tiger sneakers (the modern Mexico-66 cut; she has owned variations of these for 28 years), carrying a black leather Ystoki Tokyo briefcase + a small Tumi rolling carry-on. Her hair is shoulder-length, mostly black with a few strands of grey she does not dye. Her oyatie identity passkey is provisioned on her **iPhone 15 Pro Max** (Mexican-spec, Japanese-language UI; she paid extra for the dual-locale setup at the CDMX Telcel store in 2024) and on her **MacBook Pro M4 16"** (Space Black, 64 GB RAM, 2 TB SSD).

She clears immigration at the **Special Permanent Resident** counter — she is a Japanese national so the home-country line moves fast — at 22:24 JST, picks up her carry-on (she only checks bags for trips longer than 8 days; this is a 5-day trip), and takes the Keikyu Limited Express from Haneda Airport Terminal 3 station to Sengakuji at 22:42 JST, then transfers to the Toei Asakusa line one stop to her hotel. The **Mitsui Garden Hotel Tamachi** is a 4-minute walk from Tamachi station's Mita exit. She checks in at 23:18 JST. Room 1247 on the 12th floor (a coincidence she notices — same floor as her CDMX office). She showers, opens the in-room kettle, makes a cup of Yamamotoyama hojicha from the welcome tray, and reviews the next day's customer agenda on her MacBook for 30 minutes. She sleeps at 24:48 JST.

## §1 — Monday May 11 — customer day

Akira spends Monday in three customer meetings:

- **09:00–11:42 JST** — **Komatsu Ltd.** headquarters at Akasaka 2-3-6 Minato-ku, Tokyo. She meets the Komatsu CIO **Watanabe Kenji-san** (no relation; common name) — 58, Komatsu lifer since 1991, oversees Komatsu's global IT infrastructure including the dozen autonomous-haul-truck (AHT) deployments at Komatsu's mining customer sites in Chile + Australia + Indonesia + Mongolia that use Aurelia's fleet-coordination platform. Watanabe-san is polite + reserved + extremely knowledgeable; he asks about the v4 cutover (j167; he has read the cohort-gate report) and about the APAC-Tokyo SEV-2 (which affected Komatsu's Indonesia-Grasberg-mine deployment by 18 min of degraded read latency). Akira answers both directly in Japanese.
- **13:00–15:30 JST** — **Daifuku Co., Ltd.** R&D center at Komaki (Aichi). Akira takes the JR Tokaido Shinkansen from Shinagawa at 11:42 → Nagoya at 13:18 → Meitetsu Komaki line at 13:24 → Komaki R&D center at 13:48. She meets the Daifuku R&D Director **Inoue Sachiko-san** — 47, MIT-Sloan-classmate-via-coincidence (they overlapped 2003-2005 though they didn't know each other then; they discovered the overlap at an AGV-industry-event in 2023). They walk the Daifuku robotics test floor for 90 minutes; Daifuku is a Japanese warehouse-automation leader and uses Aurelia's robot-fleet substrate for cross-customer reference architectures.
- **17:00–18:30 JST** — **THK Co., Ltd.** Tokyo HQ at Nishi-gotanda 3-11-6, Shinagawa-ku. THK is a linear-motion-bearing manufacturer; their fleet integration is small (3 AGVs at one R&D facility) but THK's CEO **Teramachi Akihiro-san** insisted on meeting Akira personally during her Tokyo trip. He is 76, the third-generation leader of THK (founded by his grandfather Teramachi Hiroshi in 1971), known for being formal and exacting. The meeting is brief + formal; he asks if Aurelia plans to add **TrueTime-equivalent fence support for Japanese-Internet-Time (JST-NICT-attested)** for cross-tenant transactions; Akira says yes, ADR-0252 supports JST-NICT integration; he nods.

Akira takes the Keikyu back to Tamachi at 19:12 JST. She has dinner alone at a small **soba-ya** near her hotel (the chef is from Niigata, the soba is genmai-buckwheat, very dark; she has zaru-soba + duck-broth dipping sauce + small atsukan sake). She writes brief notes in her Moleskine (small black A6, the same series she has used since MIT Sloan) about each meeting in Japanese-English-mixed shorthand. She sleeps at 22:48 JST.

## §2 — Tuesday May 12, 09:00–17:42 JST — internal prep with Hiroshi Takei

Tuesday morning Akira walks the 7 minutes from the hotel to the **Aurelia-Japan office** at the **Tamachi Mitsui Building** 5th floor (1-22-23 Mita, Minato-ku). The office is small — 24 employees, half engineers half customer-success — and has the typical Japanese-office stark fluorescent + light-grey-carpet aesthetic. Her deputy **Hiroshi Takei** is waiting at the front desk at 08:52 JST.

Hiroshi Takei — 45, Aurelia's APAC-Tokyo cell operations director — joined Aurelia in **August 2023** after 19 years at **NTT Data Tokyo** + a 4-year detour at **AWS Japan Solutions Architect** team. He is from Sendai (Miyagi prefecture), Tohoku University engineering graduate 2002. Married, two daughters (10 + 13). Lives in Setagaya. Speaks Japanese (native), English (C1), Korean (B1; he did 18 months at NTT Data Seoul 2014-2015). His tenant identity reads `hiroshi.takei@aurelia-robotics-internacional-sa-de-cv-mx` with `preferred_locale: ja-JP`. He is calm, methodical, the kind of operations leader who runs a tight NOC.

**Takei 09:00 JST** (Japanese, polite-business register): "渡辺さん、おはようございます。フライト、お疲れさまでした。" *("Watanabe-san, ohayou-gozaimasu. Furaito otsukare-sama deshita." Good morning Watanabe-san. The flight must have been tiring.)*

**Akira 09:00 JST** (Japanese): "おはよう武井さん。フライトは大丈夫だった、ただ着陸が遅れた。今日は何時から?" *("Ohayou Takei-san. Furaito wa daijoubu datta, tada chakuriku ga okureta. Kyou wa nanji kara?" Good morning Takei-san. The flight was fine, just landing was delayed. What time are we starting today?)*

**Takei 09:01 JST**: "9時15分から会議室Aで。SEV-2のタイムラインを最初に。" *("Kuji jugofun kara kaigishitsu A de. SEV-2 no taimurain wo saisho ni." 9:15 in meeting room A. SEV-2 timeline first.)*

They settle in Meeting Room A (small, 6-person, glass-walled, looks out onto the building's central atrium). Akira places her MacBook + iPad on the table. Takei pulls up the `ops-dashboard-control-center` µservice's Q4-2026 dashboard on the room's wall display + on his ThinkPad X1 Carbon.

The Q4-2026 quarterly ops dashboard URL: `https://ops.aurelia-robotics-internacional-sa-de-cv-mx.oyatie.cloud/quarters/Q4-2026/snapshot`. The top-of-page shows:

```
Q4-2026 (Apr 1 - Jun 30 2026) · Quarterly Operations Snapshot
Generated: 2026-05-12T09:00:00+09:00 (08:00 local; Tokyo Asia/Tokyo)
Tenant: aurelia-robotics-internacional-sa-de-cv-mx
9 cells × 27 metrics = 243 snapshot cells

Headline metrics (cell-weighted):
  p99 latency:           94ms  (target ≤ 100ms)  GREEN
  Throughput:            418K req/sec sustained  GREEN
  Error budget burn:     0.74× (target ≤ 1.0×)   GREEN
  Capacity util:         62%   (target 50-75%)   GREEN
  Headcount per AZ:      14.2 (target ≥ 12)      GREEN
  NPS:                   68    (target ≥ 65)     GREEN
  On-call burnout:       4.8/10 (target ≤ 5.0)   GREEN (but APAC-Tokyo 6.8 RED)

Active incidents this quarter:
  SEV-1: 0
  SEV-2: 1 (incident-j168-sev2-apac-tokyo-2026-04-15-cell-failover-cascade-001)
  SEV-3: 4
```

**Takei 09:18 JST**: "全体的に緑ですが、APAC-Tokyoだけ赤い。On-call burnout 6.8/10 — チームの疲労が出ている。" *("Zentai-teki ni midori desu ga, APAC-Tokyo dake akai. Burnout 6.8/10 — chiimu no hirou ga dete iru." Overall green, but APAC-Tokyo alone is red. Burnout 6.8/10 — team fatigue is showing.)*

**Akira 09:19 JST**: "SEV-2の影響だね。タイムラインを見せて。" *("SEV-2 no eikyou da ne. Taimurain wo misete." That's the SEV-2 impact. Show me the timeline.)*

Takei opens the incident record. The incident timeline:

```
incident-j168-sev2-apac-tokyo-2026-04-15-cell-failover-cascade-001

Trigger:      2026-04-15T03:42:18+09:00 — apac-tokyo-az-a primary node OS panic
Detection:    2026-04-15T03:42:24+09:00 (6 sec) — observability alert
Failover initiated: 2026-04-15T03:42:42+09:00 (24 sec from trigger)
Failover target: apac-tokyo-az-b (intended) BUT pods landed on apac-tokyo-az-a due to anti-affinity rule misconfig
Cascade detected: 2026-04-15T03:48:18+09:00 — read latency p99 spike from 92ms to 1842ms
Failover RE-targeted: 2026-04-15T03:54:42+09:00 to apac-tokyo-az-c (manual intervention by on-call Watabe-san)
Read latency partial recovery: 2026-04-15T04:02:18+09:00 (read p99 384ms)
Read latency full recovery: 2026-04-15T04:29:42+09:00 (read p99 96ms)
Incident closed: 2026-04-15T05:48:42+09:00

Total degraded duration: 47 minutes
Affected tenants: 12% of all tenants on apac-tokyo cell = 11 of 92 active tenants
Customers severely affected: Komatsu (Indonesia Grasberg mine), Sumitomo Heavy Industries (Yokohama plant), Mitsubishi Logistics (Shinagawa DC)
Service credits issued: MXN 312,000 (per SLA — 1.5x the standard credit because credit-eligibility threshold was 30min)
Initial root-cause hypothesis: Kubernetes anti-affinity rule misconfigured to AZ boundary instead of node boundary
Initial corrective action: change anti-affinity rule from `topologyKey: kubernetes.io/hostname` to `topologyKey: topology.kubernetes.io/zone`
```

**Akira 09:42 JST**: "なるほど。深掘りは明日だね。今日は四半期メトリクスをセル別に。" *("Naruhodo. Fukabori wa ashita da ne. Kyou wa shihanki metrics wo cell-betsu ni." I see. Deep-dive is tomorrow. Today is per-cell quarterly metrics.)*

They spend the rest of Tuesday walking through the per-cell quarterly snapshot. Each of the 9 cells × 27 metrics requires Akira to read + confirm + (where she has questions) ask Takei to drill down. By 17:42 JST they have walked all 243 cells. Takei seals the metric snapshot in the audit-chain: `EVT-J168-Q4-METRIC-SNAPSHOT-001` at 17:42:18 JST.

Akira walks back to the hotel at 18:00 JST. She has dinner with her CDMX colleague **María José Hernández** (who happens to be in Tokyo on a separate Aurelia-customer-success trip; they coordinated dinner) at a small Italian-Japanese fusion restaurant in Roppongi. They speak Spanish.

## §3 — Wednesday May 13, 09:00–18:00 JST — Q4-2026 ops review

Wednesday is the formal Q4-2026 ops review. The full Aurelia-Japan office leadership attends + Diego Vargas + Yamilet Solís dial in from CDMX (their clocks are 19:00 CDT Tuesday → 09:00 JST Wednesday; they have 1 hour overlap before they need to sleep) + Brian Tate dials from Austin.

**Morning (09:00–12:00 JST)**: headline metrics + cellular-topology summary. Akira walks the room through the dashboard. Diego asks one question (about the v4 cutover impact on the latency baseline, since the cutover spanned Q4); Yamilet answers. Brian asks about NPS by customer segment; Hiroshi answers. The meeting is calm + methodical.

**Lunch (12:00–13:00 JST)**: simple bento boxes from the building's basement food hall. Akira eats with Hiroshi + 3 of his direct reports.

**Afternoon (14:00–18:00 JST)**: per-cell + per-AZ deep dive. The 9 cells × 3 AZs = 27 AZ-level summaries. Each AZ has its own latency + throughput + capacity + headcount slice. The APAC-Tokyo cell gets 90 minutes of attention because of the SEV-2; Hiroshi walks the room through each AZ's baseline + the SEV-2 deviation + the post-incident recovery curve. Diego asks 4 detailed questions about the cell-failover controller's logic. Yamilet asks 2 questions about the observability AZ-boundary readiness check (this is where the engineering corrective-action will land tomorrow). Brian asks 1 question about customer-NPS recovery curve (it's improving but not yet at pre-incident baseline).

By 17:42 JST the per-cell deep dive is complete. Akira closes Q4-2026 ops review part 2. `EVT-J168-Q4-REVIEW-COMPLETE-001a` seals.

She goes to dinner with Takei + 2 of his direct reports at a kushikatsu place in Shinbashi. They speak Japanese throughout. Akira asks Takei about his daughters' school year; Takei asks Akira about her husband Esteban (who is on a Mexico City architectural-restoration project at the historic Palacio de Bellas Artes and could not travel with her).

## §4 — Thursday May 14, 09:00–17:42 JST — SEV-2 debrief day

Thursday is the day of the SEV-2 debrief. The meeting room is now the larger Meeting Room C (12-person). Attendees:

- Akira (chair, facilitator)
- Hiroshi Takei (APAC-Tokyo cell ops director)
- **Watabe Toshio-san** — the on-call engineer who responded that night; 31, joined Aurelia in 2024, lives in Edogawa-ku
- **Kazumi Tanaka-san** — Aurelia-Japan SRE lead; 38, former Mercari SRE, joined Aurelia 2022
- **Ito Hideki-san** — Aurelia-Japan customer success lead for the 11 affected tenants
- Diego Vargas (CTO, dialed in from CDMX; 19:00 CDT Wednesday his time)
- Yamilet Solís (VP-Eng, dialed in)
- Sofía Ramírez (NOC-QRO; dialed in)
- **Brian Tate** (SVP-CS; dialed in)

The `incident-management` µservice's debrief workflow opens at 09:00 JST. The 5-Whys form initializes:

```
incident-j168-sev2-apac-tokyo-2026-04-15-cell-failover-cascade-001
Debrief lead: Akira Watanabe (COO)
5-Whys analysis:

Why 1: Why did the SEV-2 happen?
   The cell-failover controller scheduled the failover-target pods on the SAME
   AZ as the failed primary, causing cascade failure.

Why 2: Why did the failover controller schedule pods on the same AZ?
   The Kubernetes anti-affinity rule was set to `topologyKey: kubernetes.io/hostname`
   (node-level) instead of `topology.kubernetes.io/zone` (AZ-level).

Why 3: Why was the anti-affinity rule set to node-level?
   The rule was originally written in 2023 when apac-tokyo cell had only 1 AZ.
   When AZs b + c were added in Q1-2026, the anti-affinity rule was not updated.

Why 4: Why was the anti-affinity rule not updated when AZs were added?
   The cell-topology-expansion runbook in 2024-Q1 did NOT include a step
   to audit anti-affinity rules against the new AZ topology.

Why 5: Why did the runbook not include this audit step?
   The runbook was written before the substrate's cellular architecture (ADR-0248)
   was fully formalized; the runbook author assumed anti-affinity rules would
   "just work" with the new topology because they tested locally without the
   topology-aware scheduler installed.
```

The 5-Whys conversation runs from 09:00 JST to 12:42 JST. Watabe-san walks the room through his on-call experience that night (he was paged at 03:42:42; he was sleeping; he reached the laptop by 03:45:18; he saw the cascade by 03:48:00; he started typing the manual failover-retarget by 03:50:18; he had to wait for `kubectl` cluster-context auth flow to complete; the retarget executed at 03:54:42). Tanaka-san walks the room through her SRE-postmortem investigation (she identified the anti-affinity misconfig at 06:18 JST that morning; she confirmed the secondary root-cause — observability AZ-boundary check gap — at 08:42 JST).

After lunch (Italian sandwiches from the building's deli; they keep working through), they define the **corrective-action items**. 87 items across 4 engineering teams:

- **Cell-Topology Team (28 items)**: refactor anti-affinity rules for all 9 cells; update cell-topology-expansion runbook with AZ-boundary audit step; add CI-lint for anti-affinity rules; add automated test for failover correctness
- **Observability Team (22 items)**: add AZ-boundary failover-readiness check; add cell-level + AZ-level + node-level redundancy verification metric; add pre-failover dry-run mode; refactor the alerting threshold logic
- **Platform Team (19 items)**: update Kubernetes operator to enforce topology-aware scheduling by default; deprecate node-level anti-affinity for production workloads; add policy-engine guard against misconfigured topology keys
- **Incident-Management Team (18 items)**: update runbook with the manual-retarget shortcut; add the cluster-context auth flow shortcut for on-call laptops; add quarterly anti-affinity-audit task; update SEV-2-debrief training module

Each corrective-action item carries: title, owner, estimated engineering-hours, target-completion-date, dependency-chain. The aggregate engineering-hours: **3,840 hours**. At Aurelia's blended engineering rate of MXN 3,124/hour (loaded), this maps to **MXN 11,996,160 ≈ MXN 12M** — which is the capex line item that will go to Monday's Cedar gate.

By 17:42 JST the debrief is complete. `EVT-J168-DEBRIEF-COMPLETE-003` seals. The 87 corrective-action items are materialized in the `tasks` µservice; each has a draft state pending capex approval Monday.

Akira asks Hiroshi to stay 30 more minutes. They sit quietly. She thanks him in Japanese for running a tight NOC; she thanks Watabe-san for the calm response that night; she thanks Tanaka-san for the thorough postmortem. None of them work overtime today — Hiroshi insisted on this — and they all leave by 18:18 JST.

Akira walks back to the hotel. She buys a small bouquet of stargazer lilies from a Family Mart florist (she will leave them on Hiroshi's desk Friday morning as a quiet thank-you — a Japanese workplace gesture that Mexicans would find understated but that Hiroshi will read correctly).

## §5 — Friday May 15, 09:00–15:00 JST — customer relationship repair

Friday is customer-relationship-repair day. Three meetings:

- **09:30–11:00 JST** — **Komatsu Ltd.** at Akasaka HQ again (same building as Monday). She meets Watanabe-Kenji-san for a second time this week, but now with the full debrief evidence + corrective-action list. The MX 312k service-credit allocation includes MXN 84k for Komatsu's Indonesia-Grasberg deployment. Akira walks Watanabe-san through the 5-Whys + the 87 corrective actions in Japanese; he asks 4 questions; he signs the customer-relationship-repair attestation form in the room. `EVT-J168-CUSTOMER-REPAIR-004a` seals at 10:54 JST.

- **12:30–14:00 JST** — **Sumitomo Heavy Industries Ltd.** at Yokohama HQ (Sotetsu Yokohama Tower, Yokohama-shi, Kanagawa). She takes the Tokaido line from Shinagawa to Yokohama (28 min). She meets their **CIO Yamamoto Akira-san** (no relation; common name) — 56, a 30-year Sumitomo lifer. Their affected deployment was the Yokohama plant's robot fleet (37 robots, all degraded read latency during the 47-min window). Service-credit MXN 96k. Akira walks the 5-Whys again. Yamamoto-san signs at 13:54 JST. `EVT-J168-CUSTOMER-REPAIR-004b` seals.

- **15:30–17:00 JST** — **Mitsubishi Logistics Corp.** at Shinagawa HQ. She takes the Tokaido back to Shinagawa (28 min) + a 6-minute walk to their building. CIO **Suzuki Daisuke-san** — 49, formerly at Mitsubishi Corp central, joined Mitsubishi Logistics in 2018. Service-credit MXN 132k (largest of the 3 because Mitsubishi Logistics's Shinagawa DC fleet is largest; 64 robots). Suzuki-san is more pointed than Watanabe-san or Yamamoto-san; he asks 7 questions including some sharp ones about whether the cell-topology-expansion runbook gap would be caught earlier next time. Akira answers each directly. He signs at 16:42 JST. `EVT-J168-CUSTOMER-REPAIR-004c` seals.

After Mitsubishi Logistics, Akira returns to the Aurelia-Japan office one last time to thank Hiroshi + his team in person. She gives Hiroshi a small gift wrapped in furoshiki — a small bottle of Mexican mezcal (Real Minero Largo, from Oaxaca, that her husband sent her with) — and a hand-written thank-you note in Japanese using her fountain pen (a Pilot Custom 823 with Iroshizuku Yama-budou ink). Hiroshi bows + accepts both with both hands.

Akira leaves the office at 17:42 JST. She catches the Keikyu to Haneda at 18:18, makes the 19:48 JST departure of ANA NH172 (Tokyo → Houston → Mexico City), and arrives at MEX at **05:42 CDT Saturday** May 16. She sleeps in the airport's premium lounge for 90 minutes, then takes a taxi home to Lomas de Chapultepec where Esteban + their cat (a Japanese-bobtail named Nami) are waiting.

## §6 — Sat May 16 + Sun May 17 — CRA drafting at home

Saturday + Sunday Akira drafts the Q1-2027 OKR + capex CRA document. She works from her home office (a converted guest bedroom on the second floor of their 1962 Le-Corbusier-influenced Lomas-de-Chapultepec house). She uses her MacBook + iPad + the same Moleskine notebook she carried in Tokyo. The CRA covers:

- **Q1-2027 OKR cycle objectives** (3 company-level OKRs):
  1. Maintain v4 platform p99 latency ≤ 90ms cell-weighted (key result: 90% of cells)
  2. Reduce on-call burnout cell-weighted to ≤ 4.0/10 (target by Q1-2027 end)
  3. Expand to 3 new customer regions (key result: 3 cells live in {Brazil-Rio, India-Mumbai, Vietnam-Hanoi})

- **Q1-2027 capex line items** (12 lines totaling MXN 218M):
  - Line 1: SEV-2 corrective-action engineering — **MXN 12M** (the 87 items)
  - Line 2: 3 new region cells (capex per cell ≈ MXN 38M × 3) — **MXN 114M**
  - Line 3: Observability platform refactor — MXN 18M
  - Line 4: AI/ML path-planning module upgrade — MXN 24M
  - Line 5: Customer-success tooling (Japanese + Portuguese + Vietnamese localization) — MXN 8M
  - Line 6-12: Misc smaller items totaling MXN 42M

She signs the CRA Sunday evening at 18:42 CDT via QES (sat-mx-FIEL). `EVT-J168-CRA-SIGNED-006` seals.

She has dinner with Esteban at their house (he made chicharrón en salsa verde + tortillas hechas a mano). They talk about her trip; he tells her about his Palacio de Bellas Artes restoration project. Nami sits in his lap. They go to bed at 22:42 CDT.

## §7 — Monday May 18, 09:00–11:42 CDT — capex Cedar quorum

Monday morning Akira is at her Torre Manacar 12th-floor office at 07:42 CDT. She has espresso + reviews the CRA one final time. At 08:42 CDT she walks two floors up to Diego's office; they chat briefly + walk down together to the 11th-floor large conference room.

The capex Cedar quorum opens at 09:00 CDT. Quorum members:

1. **CEO Hugo Ávila Mendoza** — 58, founder + CEO since 2017; engineering background (PhD MIT Mech-E 1996); joined the room at 08:58 CDT
2. **COO Akira Watanabe** — herself
3. **CTO Diego Vargas** (j167)
4. **CFO Patricia Carrillo Vega** — 53, joined from Banorte CFO seat in 2023; IFRS + Mexican-corporate-finance specialist
5. **Board Operations Committee Chair Patrick Reilly** — 64, US-Irish, retired Cisco-Systems VP, board member since the 2024 IPO; joined via Webex from Tahoe (he's spending the spring at his lake house)

The Cedar quorum modal opens. Each line item is voted independently (this is a Cedar policy precondition for line items > MXN 5M; the 8 smaller line items are bulk-approved with a single 5-of-5 vote).

For Line 1 (SEV-2 corrective-action MXN 12M):

- CEO Hugo votes PERMIT 09:18:18 CDT with rationale (Spanish): "Aprobado. La causa raíz está clara y las acciones correctivas son específicas y medibles."
- COO Akira votes PERMIT 09:18:42 CDT with rationale (Spanish): "Aprobado. El análisis 5-Whys es sólido. Los 87 items son específicos y los plazos son razonables."
- CTO Diego votes PERMIT 09:19:18 CDT: "Aprobado. El gap topológico es real y la corrección es la correcta."
- CFO Patricia votes PERMIT 09:19:42 CDT: "Aprobado. Los MXN 12M son razonables para 3,840 horas de ingeniería al rate cargado actual."
- Board-Ops-Chair Patrick votes PERMIT 09:20:18 CDT (English): "Approved. The corrective-action plan is comprehensive and the SEV-2-incident attribution to capex is appropriate."

5-of-5 PERMIT. Line 1 sealed: `EVT-J168-CAPEX-LINE-1-PERMIT-007a` at 09:20:42 CDT under TrueTime fence (uncertainty 1.8 ms).

The 3 new-region cells (Lines 2a, 2b, 2c) each get a separate 5-of-5 PERMIT vote. Each cell is MXN 38M, exceeds the MXN 5M threshold, requires its own quorum.

Lines 3-7 (observability refactor MXN 18M; AI module upgrade MXN 24M; customer-success tooling MXN 8M; two more individual lines > MXN 5M each) each get individual votes.

The 8 smaller items (totaling MXN 42M, all individually < MXN 5M) get bulk-approved with one 5-of-5 vote.

Total: 9 individual votes + 1 bulk vote = 10 quorum decisions. By 11:42 CDT all 10 are sealed. Aggregate audit seal: `EVT-J168-CAPEX-PERMIT-007` at 11:42:18 CDT under TrueTime fence.

The 87 corrective-action items in `tasks` µservice transition from `draft_pending_capex` to `funded` state. The engineering work formally begins this week.

`EVT-J168-CAPEX-LINKED-008` seals at 11:42:30 CDT linking the SEV-2 corrective-action line to the original SEV-2 incident record.

The Q4-2026 ops review report auto-generates at 11:48 CDT (the workflow waited for the capex link before generating). The report is sent to:

- PwC México (SOC2 evidence + ISO-22301 evidence)
- KPMG México (IFRS-15 service-credit deduction evidence — the MXN 312k credit deducts from Q1-2026 already-recognized revenue)
- DEKRA Certification GmbH (EU-AI-Act-Art-19 post-market monitoring evidence — the SEV-2 affected the path-planning AI module's runtime; DEKRA needs the post-market monitoring report within 30 days per EU-AI-Act Article 19)

`EVT-J168-REPORT-SUBMITTED-009` seals at 11:54 CDT.

Akira closes her MacBook at 12:18 CDT. She goes to lunch alone at the building's Italian restaurant on the ground floor. She orders a small spinach-ricotta ravioli + sparkling water + an espresso. She reads on her iPad — a Japanese novel (Yuko Tsushima's *Territory of Light* in the original Japanese) — for 30 minutes. She walks back upstairs at 14:00 CDT to start the next thing.

## §8 — Beats not on the wire (the human texture)

- During Tuesday's internal prep, Akira noticed that Watabe-san — the on-call engineer who responded the night of the SEV-2 — was visibly tense even now, a month after the incident. She wrote a personal note in her Moleskine: "確認: 渡部さんが大丈夫か個別に話す" *(check in with Watabe-san one-on-one)*. On Friday afternoon between the Sumitomo + Mitsubishi meetings she took 20 minutes to sit with him in the office's small kitchenette. They talked in Japanese. He admitted he had felt personally responsible for the cascade; he had been replaying his 12-minute decision sequence in his head. Akira told him directly: "渡部さん、あなたは何も間違っていない。問題はランブックの欠陥にあった。あなたの対応は正しかった、ただランブックがあなたを助ける形になっていなかった。" *("Watabe-san, you did nothing wrong. The problem was the runbook gap. Your response was correct; the runbook simply wasn't built to help you.")* He nodded; his shoulders relaxed a little. She added that he is among the on-call engineers being assigned to the corrective-action work — specifically the runbook + cluster-context-auth-shortcut items — so he can directly shape the fix.
- The stargazer lilies on Hiroshi's desk Friday morning were a quiet gesture that the rest of the Aurelia-Japan office understood. By lunchtime three engineers had asked Hiroshi who they were from; he just said "From Akira-san. She's thanking us." He kept them on his desk for the next 8 days.
- Akira's husband Esteban is a working architect; he has been on the **Palacio de Bellas Artes interior-restoration** project for 11 months. He had wanted to travel to Tokyo with her but the Palacio's structural-engineer consultation week was scheduled for the same Tuesday/Wednesday. He sent her a voice-note on Wednesday evening (her time): "Akira, mi amor. Te extraño. Te mando un abrazo desde el Palacio. Esteban." She replied in voice-note in Spanish: "Esteban, gracias. Te veo el sábado. Cuida a Nami por mí."
- Diego Vargas — the CTO from j167 — and Akira have a quiet professional rapport. They don't socialize outside work much (different generational cohorts — Diego 47, Akira 51), but they trust each other. During the Wednesday ops review, when Diego asked his fourth detailed question about the cell-failover controller's logic, Akira noticed that Hiroshi (whose English is fluent but not effortless) was struggling slightly with the technical depth + the speed. She gently interrupted in English: "Diego, let me ask Hiroshi to walk us through this in Japanese first, then I'll translate the key points. The detail matters and I want to make sure the team here can give a precise answer." Diego said "Of course Akira" and waited. Hiroshi answered in Japanese for 4 minutes; Akira translated; the answer was clearer.
- Akira's mother — **Watanabe Sachiko** — is 78, lives in Yokohama (Hodogaya-ku), retired from her career as a high-school English teacher. Akira had planned to visit her mother on Saturday morning before flying back. The Friday-night flight back disrupted this; she called her mother from the Haneda lounge at 19:18 JST instead. They talked for 14 minutes in Japanese — about the weather (rainy season starting in Yokohama), about Esteban's project, about Akira's cat Nami, about Sachiko's grandson (Akira's older brother's son, who is 19 and studying at Waseda). Sachiko said "次回はもう少し長く居て" *("Next time stay a bit longer")*. Akira said yes, mother, next quarter.
- The Mitsubishi Logistics CIO Suzuki-san's sharp questions Friday afternoon left Akira thinking through the weekend. On Sunday she added a 13th OKR-cycle action-item to her Moleskine: "Q1-2027: build customer-facing transparency dashboard for SEV-incident root-cause-and-corrective-action; let Tier-1 customers see corrective-action progress live." She wrote this in Japanese-English mixed shorthand. The item is not in the Q1-2027 capex (she hadn't drafted it yet by Sunday evening when she signed the CRA) but she will add it to the Q2-2027 capex pipeline.
- The night-cleaning crew at Torre Manacar floor 12 — same Limpieza Profesional CDMX subcontractor as j167 floor 14 — saw Akira's office light on Sunday at 21:42 CDT. The lead cleaner doña Refugio Pérez (who appeared in j167) didn't bring her coffee this time; she just left a small bowl of mints on the corner of Akira's desk. Akira saw them when she packed up at 22:18 CDT. She left the mints + took 2 with her in her pocket.

## §9 — Stop condition for this story

This story documents the 6-day journey from Akira's CDMX-Tokyo flight through her Q1-2027 capex Cedar quorum. The acceptance criteria in `README.md`, the API shapes in `handshake.md`, the test cases in `integration-test-plan.md`, and the schema files together encode the machine semantics. The story exists so that the next human or agent reading the codepath can understand WHY the quarterly ops dashboard materializes 9 cells × 27 metrics with audit-chain Merkle attestation, WHY the SEV-2 debrief's 5-Whys form is a first-class workflow artifact linked to corrective-action tasks linked to capex line items, WHY the customer-relationship-repair meetings dual-seal as cross-tenant attestations (so the customers can replay the evidence chain themselves), WHY the capex Cedar quorum is 5-of-5 PERMIT for line items > MXN 5M with TrueTime fence ≤ 10 ms, and WHY a Japanese-Mexican COO running APAC + AMER + EU regions can land a quarterly ops review with reproducible evidence in 6 days rather than 6 weeks because the substrate gives her audit-attested metric infrastructure instead of PowerPoint-and-Excel summaries.
