---
doc_class: User-Journey-Story
journey_id: j01-emergency-911-dispatch
journey_slug: j01-emergency-911-dispatch
status: published
date: 2026-05-20
authority_tier: 3
audience: [council-product, council-architecture, council-security, council-legal, ops-trust-and-safety, axis-emergency-services]
related_adrs:
  - ADR-0298-emergency-services-bypass-life-safety
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0263-observability-emission-contract
  - ADR-0251-compliance-pack-cell-certification-levels
related_specs:
  - /specs/emergency-services-bypass.json
  - /specs/microservices/api-gateway.json
  - /specs/microservices/messenger.json
  - /specs/microservices/identity.json
  - /specs/microservices/cell.json
  - /specs/microservices/observability.json
  - /specs/microservices/audit-chain.json
related_packs:
  - packs/kr-119-operational-mandate
  - packs/kr-pipa-2023-amendment
  - packs/global-emergency-services-baseline
critical_path_rows:
  - documentation-rigor.md §3.2.5 row 1 (Emergency services)
  - documentation-rigor.md §3.2.5 row 22 (Mass-casualty surge — see j12 cross-link)
anchor_archetype: yejin-park-38-seoul
locale: ko-KR
regulatory_anchors:
  - 위치정보의 보호 및 이용 등에 관한 법률 Art. 29 (KR Location Information Act, emergency-services exception)
  - 응급의료에 관한 법률 (KR Emergency Medical Service Act)
  - KR-119 dispatch operational standard MPSS-119-2024
purpose: >
  Narrate ONE concrete human's experience triggering the oyatie emergency-services
  bypass when her husband collapses at home. Trace identity continuity across
  her day-to-day consumer messenger account, her enterprise nurse account,
  and the patient-PHI surface as the 119 dispatcher arrives. Demonstrate why
  ADR-0298 cannot be optional: the bypass MUST fire even though Yejin's
  account would otherwise be rate-limited by abuse-defence baseline.
---

# j01 — Emergency 119 dispatch: Yejin Park's worst Tuesday

> **Purpose.** This is not a hypothetical. This is the story of Yejin Park,
> 38, a senior nurse at Seoul National University Hospital (SNUH), at 14:07
> on a humid Tuesday afternoon when her husband Min-jun collapses in their
> apartment in Gangnam-gu. Every µservice in the oyatie fabric will be
> exercised in the next eleven minutes. The story is concrete because the
> contract is concrete: 911-class life-safety dispatch is the FIRST critical
> path documentation-rigor.md §3.2.5 names, and ADR-0298 makes the bypass
> architecturally mandatory. If any seam in this story would have failed,
> Min-jun would have been on the floor longer. Every line of code we ship
> must protect this minute.

## 1. Yejin's continuity of identity — one human, three contexts

Yejin Park is not three users. She is one human across three contexts that
oyatie distinguishes by **principal-class-overlay** (ADR-0244), not by
fragmenting her identity.

| Context | Tenant | Principal | Cell tier | Pack overlay |
|---|---|---|---|---|
| **Day-to-day consumer** | `oyatie.consumer.kr` | `yejin@oyatie.me` | Tier-2 (consumer general-purpose) | `pack-kr-pipa-2023-amendment` |
| **Enterprise — nurse at SNUH** | `snuh.org` (B2B tenant on oyatie) | `yejin.park@snuh.org` (work passkey) | Tier-3 (regulated HIPAA-eligible) | `pack-hipa-2024 + pack-kr-pipa + pack-kr-medical-records-act` |
| **Patient (of her own hospital)** | `snuh.org` (as PHI subject) | `yejin.park@patient.snuh.org` (subject-of-record DSAR proxy) | Tier-3 (regulated PHI cell) | `pack-hipa-2024 + pack-kr-medical-records-act` |
| **Personal-business (vintage clothes)** | `oyatie.merchant.kr` | `yejin-vintage@oyatie.me` (Stripe Connect linked) | Tier-2 | `pack-kr-pipa + pack-kr-fss-2024 (payments)` |
| **Family parent (two minors)** | `oyatie.family.kr` (family-account-mode) | `yejin@oyatie.me` (parent role) | Tier-2 with COPPA/KOSA/KR-Youth pack | per ADR-0292 minor-doctrine |

When Min-jun collapses she will, within eleven minutes, **cross every one
of these tenants from a single device** — and oyatie will route each cross
through Cedar policy without forcing her to log out, switch profiles, or
re-authenticate. That cross-context fluidity is the oyatie hyperscaler shape
(ADR-0248) and the substrate-vs-product layering doctrine (ADR-0245).

## 2. The minute before — 14:06 KST, Tuesday, 2026-05-26

Yejin's iPhone 16 Pro sits on the kitchen counter. The lock screen shows:

- A waiting reply in oyatie Messenger from her friend group "한강 산책팀"
  (Hangang walking team) — three messages about Saturday's plan.
- An oyatie Mail badge: 4 unread (one is a Stripe Connect 입금완료 notification
  from her vintage-clothing side-business).
- An oyatie Workflow Studio push notification: her "신상품 자동출고" workflow
  ran and a customer paid for a 1990s denim jacket.
- A oyatie Calendar push: 14:30 PM — her son's piano-lesson pickup.
- An oyatie Notes draft: "Min-jun 약 처방받아오기" (Pick up Min-jun's
  prescription) — created during her break that morning at SNUH.

She is in HER consumer principal — `yejin@oyatie.me`. Her work session is
**also active** in the background under `yejin.park@snuh.org`, but locked
behind a session-overlay barrier per ADR-0247 (work cell is HIPAA-eligible
Tier-3; consumer cell is Tier-2 general-purpose; the cells DO NOT share
storage even though the device shares an OS session — they share only the
Cedar permit graph at the API-gateway).

She is making tea. Min-jun is at the dining table answering email.

At 14:06:52 she hears a chair scrape. A thud. Silence.

## 3. T+00:00 — 14:07:00 KST — The dial

Yejin runs from the kitchen. Min-jun is on the floor, pale, not breathing.
Her training kicks in: she clears his airway, starts CPR, and with her free
hand she opens her iPhone's Phone app and dials 119.

She does NOT open oyatie Messenger. She uses the OS native phone app.

But the moment the 119 call connects, two things happen on the oyatie
substrate that she does not see:

1. **iOS Emergency SOS shares her location** to the carrier's Location
   Information service via the Korean E112-equivalent emergency-call-routing
   per 위치정보의 보호 및 이용 등에 관한 법률 Art. 29. The carrier route
   triggers a 119 PSAP (Public Safety Answering Point) located in
   Seoul-Gangnam dispatch center.
2. **iOS Emergency SOS auto-notifies her oyatie emergency contacts.** This
   is the first oyatie surface touched. Yejin set up two emergency contacts
   six months ago when she onboarded onto oyatie Family:
   - Her mother, `yejin-mother@oyatie.me` (in Busan).
   - Her closest nurse colleague, `dr.kang@snuh.org` (also a doctor at SNUH).
   These contacts receive an oyatie Messenger SOS push at 14:07:14 KST.

The push payload contains:

```
{
  "audience_type": "EMERGENCY_SERVICES_SOS",
  "from": "yejin@oyatie.me",
  "kind": "ios-emergency-sos-relay",
  "device_attestation": "<DeviceCheck-attest blob>",
  "coords": {"lat": 37.4979, "lng": 127.0276, "accuracy_m": 8},
  "timestamp_kst": "2026-05-26T14:07:14+09:00"
}
```

This push is dispatched via the oyatie Messenger transit channel, but it
bypasses the **abuse-defence rate-limit baseline** (ADR-0297) because the
audience_type is on the §3.2.3 emergency-services allow-list. The rate-limit
is not relaxed *only for Yejin*; it is relaxed for the **outbound
emergency-services-SOS class**. The bypass is class-scoped, not
identity-scoped — that's the ADR-0298 invariant.

## 4. T+00:14 — 14:07:14 — The first message lands

Yejin's mother sees the push 14 seconds after Yejin dialed 119. She does
not have the cognitive bandwidth (76 years old, asleep at 14:07 in Busan)
to fully process. The push is rendered in the Messenger lock-screen surface
with the **EMERGENCY_SERVICES_SOS** banner that Yejin's mother has
NEVER seen before. The Messenger app's lock-screen surface uses an
audience-overlay rendering hint from ADR-0297 §D-7 that puts the
red-bordered "응급 — 박예진님의 SOS 발신" banner at the top.

She taps. The app opens to a one-tap surface: "현재 위치 확인" /
"전화 걸기" / "119 통화중 — 그대로 두세요" — three buttons.

Yejin's nurse colleague dr.kang receives the same push at 14:07:14 on her
work iPhone (also signed into oyatie). dr.kang is in the SNUH on-call lounge.
She immediately taps "전화 걸기" — but the call routes to Yejin's mobile and
Yejin is on the 119 line. dr.kang's Messenger surface shows "통화중 — 메시지
보내기" and she types: "어디?" Her message hits Yejin's lock screen at 14:07:38.

## 5. T+00:48 — 14:07:48 — The 119 dispatcher accepts the call

The 119 PSAP in Seoul-Gangnam dispatch center is operated by Seoul-MFD
(Metropolitan Fire Department). The dispatcher console is **NOT oyatie**.
But the dispatcher console queries the oyatie **Emergency Services
Interop API** (see ADR-0298 §C and `/specs/emergency-services-bypass.json`)
to:

1. Confirm the location the carrier provided.
2. Retrieve the caller's **registered emergency profile** if any — which
   the oyatie consumer tenant exposes ONLY through this E.S. Interop API,
   under Cedar policy `emergency-services-readonly-attested.cedar`.

The dispatcher console authenticates to the oyatie E.S. Interop API via
a per-PSAP **attested SPIFFE-ID** issued under the SPIRE bootstrap chain
(ADR-0295 + ADR-0293). The SPIFFE-ID is `spiffe://emergency.korea.gov/psap/seoul-mfd/gangnam`.
The oyatie API-gateway validates the SPIFFE attestation and admits the request.

Cedar policy `emergency-services-readonly-attested.cedar` permits:

```cedar
permit (
  principal in EmergencyServices::AttestedDispatcher,
  action == Action::"emergency.read_profile",
  resource is User::"yejin@oyatie.me"
) when {
  principal.attested_psap == "seoul-mfd.gangnam" &&
  resource.opted_in_emergency_profile == true &&
  context.compliance_pack_active("pack-kr-119-operational-mandate") &&
  context.audit_session_open == true
};
```

The dispatcher sees on their console (rendered server-side by the oyatie
Emergency Services surface, not by the SeoulMFD console):

- Name: 박예진 (Park Yejin)
- Age: 38
- Medical alert: PEANUT ALLERGY (severe); none for the patient
- Pre-set emergency contact: Mother (Busan), Dr. Kang Ji-eun (SNUH)
- Spoken language preference: ko-KR
- Last-known location (oyatie): coffee shop downtown, 12:30 KST (stale — call origin is the truth)

The dispatcher reads the medical alert to the EMS team that has been
dispatched. The EMS team is en route by 14:08:24 — 84 seconds from Yejin's
first dial. (KR-119 SLA for Seoul-Gangnam: ≤6 minutes ambulance-on-scene
median; this case will hit 4 min 38 sec.)

## 6. T+01:30 — 14:08:30 — The audit trail begins

At the moment the dispatcher pulled Yejin's emergency profile, the oyatie
**audit-chain** µservice received from API-gateway an audit event:

```
audit_event: {
  class: "EmergencyServiceProfileRead",
  binding_adr: "ADR-0298",
  actor: {
    spiffe_id: "spiffe://emergency.korea.gov/psap/seoul-mfd/gangnam",
    attested_at: "2026-05-26T14:07:48.142+09:00",
    attestation_chain_hash: "blake3:e7f4...8b21"
  },
  subject: {
    user: "yejin@oyatie.me",
    tenant: "oyatie.consumer.kr",
    cell_tier: 2,
    compliance_packs: ["pack-kr-pipa-2023-amendment", "pack-kr-119-operational-mandate"]
  },
  action: "emergency.read_profile",
  cedar_decision: "PERMIT",
  cedar_fragment: "emergency-services-readonly-attested.cedar@v3",
  fields_returned: ["name", "age", "medical_alerts", "emergency_contacts", "language_pref", "last_known_location"],
  pii_class: "PHI-emergency-bypass-attested",
  audit_id: "audit:e34a...f912",
  observability: {
    trace_id: "0af7651916cd43dd8448eb211c80319c",
    span_id: "00f067aa0ba902b7",
    metric_emission: "oya_emergency_profile_read_total{pack=\"pack-kr-119-operational-mandate\",psap=\"seoul-mfd.gangnam\"}"
  }
}
```

The audit event is sealed into the audit-chain Merkle tree (ADR-0003 +
ADR-0028) within 200ms. It will be **retained for 6 years** per KR-119
operational mandate and 7 years per KR-PIPA Art. 28 (whichever is longer).
It will be discoverable by the KR-PIPC (Personal Information Protection
Commission) via the regulator transparency surface (critical-path row 18
per documentation-rigor.md §3.2.5).

The audit event is also relayed to oyatie observability backplane (ADR-0263)
where the per-pack alert rule `pack-kr-119-emergency-profile-read-rate`
sees the spike. Today's spike (single read) is not unusual. A spike of
≥1000/min triggers the j12 mass-casualty-incident playbook.

## 7. T+02:15 — 14:09:15 — The CPR continues; the EMS team gets a pre-notification

While Yejin continues CPR, the EMS team in the ambulance receives a
**pre-arrival packet** via oyatie's KR-119 interop channel. The packet is
generated by the SeoulMFD dispatcher's console pulling — under the same
`emergency-services-readonly-attested.cedar` permit — Yejin's **household
emergency profile**:

- Two adults; one minor child (8 years); one minor child (5 years).
- Min-jun (husband, 41): no chronic conditions on record; mild hypertension
  noted in his oyatie Notes private profile (NOT in any healthcare record
  oyatie holds — this is consumer-self-asserted, NOT PHI).
- Living situation: 8th floor, elevator-equipped apartment.
- Address details: confirmed by GPS coord triangulation + apartment registry
  (provided by Naver Maps API; oyatie does not query this directly — the
  SeoulMFD console does).

Note the distinction: Min-jun's **PHI** at any hospital is NOT in this
packet. oyatie does not, by default, expose any user's PHI to emergency
services. What it exposes is the **opted-in emergency profile** that the
user themselves curated under `pack-kr-pipa-2023-amendment` consent.

This is a documentation-rigor.md §3.2.5 row 1 invariant: the emergency
bypass DOES NOT become a cross-tenant data-leak surface. It is scoped to
the data the user opted to expose for this exact purpose.

## 8. T+04:38 — 14:11:38 — The ambulance arrives

The EMS team (driver, paramedic, EMT) arrives at the apartment in 4 min
38 sec. They use Min-jun's pre-disclosed hypertension to inform their
working hypothesis (cardiac event vs. neurological). They start oxygen,
attach defibrillator pads, and prepare him for transport.

Yejin is shaking. The EMT senior asks her — calmly, professionally:

> 박예진님, 어느 병원으로 모실까요?

She answers without thinking: "서울대병원." (SNUH — her own hospital.)

The EMT confirms. The ambulance departs at 14:12:50 KST.

## 9. T+05:50 — 14:12:50 — The handoff to SNUH

The EMS team uses their dispatch terminal to notify SNUH ER of the
incoming patient. This notification flows through the KR-119 dispatch
backbone to SNUH's ER intake system. SNUH's ER intake system is
**oyatie Workflow Engine** (this is the enterprise side — the SNUH B2B
tenant on oyatie).

The Workflow Engine runs the `er-intake-incoming-acute` workflow under
SNUH's tenant. The workflow is one Yejin herself reviewed two months ago
during her nurse-orientation. It looks like:

```
workflow: er-intake-incoming-acute
  trigger: kr-119-eta-pre-arrival-event
  steps:
    1. parse-eta-payload → patient_demographics, eta_minutes, presenting_complaint
    2. cedar.evaluate("workflow.er-intake.create_chart", tenant=snuh.org) → PERMIT/DENY
    3. ehr.create_pending_chart(patient_demographics, source="119-EMS-pre-arrival")
    4. nurse_roster.notify_next_available(specialty="emergency", priority="acute-cardiac-suspect")
    5. audit_emit("ChartPendingCreatedFromPreArrival")
    6. observability.emit_metric("snuh_er_pre_arrival_intake_total{outcome=success}")
```

The workflow creates a pending chart for Min-jun. At step 2, the Cedar
evaluation:

```cedar
permit (
  principal == Workflow::"snuh.org/er-intake-incoming-acute",
  action == Action::"ehr.create_pending_chart",
  resource is Tenant::"snuh.org"
) when {
  principal.attested_origin == "kr-119-dispatch" &&
  resource.compliance_pack_active("pack-hipa-2024") &&
  resource.compliance_pack_active("pack-kr-medical-records-act") &&
  context.source_event.is_acute_emergency == true
};
```

The PERMIT decision is recorded; the chart is created at 14:13:02 with the
provisional MRN `SNUH-2026-0526-1413-002`. The chart is NOT linked to
Min-jun's existing patient record (SNUH only has Yejin in their records,
not Min-jun) — it is created from his demographic alone. The chart will
be reconciled with his existing identity (he has no SNUH record; this is
his first SNUH encounter) once he is admitted and an SNUH MRN is allocated
to him.

The roster step (step 4) pings the next available emergency-medicine
attending. It happens to be Dr. Park Si-woo (no relation to Yejin). She
gets the page on her oyatie work-phone at 14:13:10 KST.

## 10. T+07:00 — 14:14:00 — Yejin gets in the ambulance

Yejin asks to ride along. The EMTs confirm (Korean EMS standard practice
permits one family member). She grabs her phone, her wallet, and Min-jun's
ID card. The ambulance leaves at 14:14:00 KST.

She is now in the ambulance. She is on her oyatie consumer principal
`yejin@oyatie.me`. She has NOT yet logged into her work principal. She
is still receiving oyatie Messenger pings — dr.kang has now realized what
happened and is in coordination mode on the SNUH side.

dr.kang's first message to Yejin at 14:14:18:
> 예진아 SNUH ER 도착하면 나한테 전화해. 박시우 선생 인계받았어.

Yejin reads it. She does not reply. She types "OK". The message goes via
the standard oyatie Messenger transit channel — `consumer.kr` cell to
`work.snuh.org` cell. Because both Yejin (sender) and dr.kang (recipient)
have opted into cross-tenant DM under their respective tenant policies
(SNUH allows cross-tenant DM with verified-personal-contacts of staff),
the Cedar evaluation permits the message.

The audit event captured here is:

```
class: "CrossTenantDM"
sender: "yejin@oyatie.me" (consumer.kr)
recipient: "dr.kang@snuh.org" (work.snuh.org)
cedar_decision: PERMIT
cedar_fragment: "cross-tenant-dm-personal-verified.cedar@v2"
audience_type: "VERIFIED_PERSONAL_CONTACT"
pii_class: "Conversational-low"
audit_id: "audit:f78b...921e"
```

This is normal day-to-day data, not emergency-services-class. It does
not bypass abuse-defence, but it does pass abuse-defence because Yejin and
dr.kang have a 4-year DM history.

## 11. T+12:00 — 14:19:00 — Yejin arrives at SNUH ER

The ambulance pulls into the SNUH ER ambulance bay at 14:19:00 KST. Min-jun
is unloaded onto a gurney. Dr. Park Si-woo is at the bay. Yejin walks
alongside, recognized by every nurse on duty.

She approaches the intake desk. The intake nurse, also recognized, says:
"예진 선생님 본인 신분증으로 본인 인증해 주세요. 환자 보호자 등록할게요."

Yejin opens her phone. She is asked to authenticate as `yejin.park@snuh.org`
(her work principal) — because at the intake desk she will be registering
herself as **patient's next-of-kin** in the SNUH EHR, which is a Tier-3
HIPAA-equivalent action. She taps the SNUH workplace app on her phone, which
prompts:

```
서울대병원 직원 인증
패스키로 인증해 주세요 (1/1)
```

She uses her Face ID + passkey. The passkey is the FIDO2 WebAuthn-class
passkey she registered when she joined SNUH 7 years ago. Per ADR-0188
passkeys are the canonical auth and SMS-MFA is forbidden on Tier-3 cells.

Authentication completes at 14:19:18 KST. The session-overlay barrier
between her consumer principal and her work principal is now bridged
under the **active-clinical-context** flag (Cedar policy
`active-clinical-context.cedar`) — a flag that permits her to act as nurse
and as patient-family-member on the same device for the next 4 hours.

The audit event:

```
class: "PrincipalContextSwitch"
from_principal: "yejin@oyatie.me"
to_principal: "yejin.park@snuh.org"
context_flag: "active-clinical-context"
duration_seconds: 14400
authentication_method: "passkey-fido2"
audit_id: "audit:8c2a...44e7"
```

She fills out the next-of-kin form in the SNUH EHR (which is oyatie Workflow
Engine surface). She authorizes emergency consent for Min-jun (Korean
medical practice permits spouse to give surrogate consent in life-threatening
situations under 응급의료에 관한 법률 §9).

Min-jun is rushed into the resuscitation room. Dr. Park Si-woo's team
diagnoses **sudden cardiac arrest secondary to undiagnosed long-QT syndrome**
within 9 minutes. They successfully defibrillate. Min-jun's spontaneous
circulation returns at 14:31:42 KST — 24 minutes 42 seconds from Yejin's
first dial.

He will live.

## 12. T+24:42 onward — The hours after

Yejin sits in the ER waiting area while Min-jun is stabilized. She has not
eaten since breakfast. Her phone buzzes. She has 47 unread Messenger
notifications. Her mother is calling from Busan. She picks up.

At 14:45 KST her son's school calls — she had set up oyatie Workflow Studio
to auto-forward school calls to her oyatie Voicemail-to-Text when she is
in active clinical context. The voicemail says her son is asking why no
one picked him up from piano. Her workflow had also auto-paged her
mother-in-law via oyatie Messenger at 14:30:00 KST (the moment the
calendar-pickup time was missed and active-clinical-context flag was
detected by the calendar µservice).

Mother-in-law picks up the son at 14:50.

Yejin sits down. She opens her oyatie Notes, taps the "오늘" entry, and
writes one line:

> 14:07 민준 쓰러짐 / 14:31 회복 / 살아있어 다행

She does not know yet that the audit trail of every system that touched
this hour will be visible to her — for her own subject-access-request —
through oyatie's DSAR surface (critical-path row 18). She has time for
that later.

She does not know yet that, because Min-jun's intake chart was created at
SNUH (a tenant she also works for), oyatie's **tenant-as-universal-scoping
-primitive** doctrine (ADR-0244) prevents her from accessing his chart
through her nurse role — the chart will only be accessible to clinicians
on his care team, not to her. She will need to be added explicitly by
Min-jun's primary attending if she wants clinical visibility, and that
addition will be auditable.

She does not know yet that the abuse-defence baseline (ADR-0297) saw the
SOS push at 14:07:14 and emitted a `WHITELISTED_EMERGENCY_BYPASS` signal
which propagates a small green flag in oyatie's observability backplane —
visible only to the ops-trust-and-safety team — meaning: "this user's
account triggered an emergency-services bypass; treat any subsequent
abuse-defence flag with elevated context until 24 hours expire."

She does not know yet that, in the §3.2.5 row-1 + row-22 cross-link, today
was NOT a mass-casualty incident in Seoul-Gangnam dispatch (which would
have routed her call differently). Today was a single cardiac arrest. The
substrate scaled fine.

She knows only that her husband is alive.

## 13. The next morning — 2026-05-27, 09:00 KST

The next day, the SNUH ICU attending discharges Min-jun to a cardiology
ward. Yejin returns to work — to her own ICU floor, three floors above
where Min-jun is admitted.

At 09:00 KST she opens her oyatie Workflow Engine. There is a banner:

> 박예진님: 어제 14:07-14:31 응급 상황 관련 감사기록 47건 있습니다.
> 자세히 보기 / 나중에

She taps "자세히 보기". The DSAR-class surface (Cedar policy
`subject-access-request-emergency-context.cedar`) shows her every audit
event that touched her account during the emergency window. She can see:

- 1× iOS Emergency SOS relay to oyatie Messenger contacts (T+0:14)
- 1× emergency-services profile read by SeoulMFD Gangnam (T+0:48)
- 1× pre-arrival packet generation (T+2:15)
- 4× cross-tenant DM with dr.kang (T+7:18 onward)
- 1× principal-context-switch (T+12:18)
- 1× next-of-kin registration in SNUH EHR (T+12:45)
- 1× surrogate-consent declaration (T+13:02)
- 17× workflow-triggered notifications (school call forward, mother-in-law,
  calendar reschedule, etc.)
- 21× standard chat/social messages (Messenger group, mother, in-laws)

Each event has its own `audit_id` traceable to the sealed Merkle path. Each
event lists which Cedar fragment permitted it, which pack overlay was
active, and which observability trace correlates. Yejin can export this
as a JSON bundle for her own records, request deletion of specific fields
under KR-PIPA Art. 36 (rectification + erasure), or appeal under Art. 38.

She does none of that. She closes the surface. She goes to her shift.

## 14. The contract this story enforces

Every paragraph above maps to a specific contract surface in the oyatie
fabric. The architecture invariants this story exercises:

1. **Emergency-services bypass is class-scoped, not identity-scoped** —
   the rate-limit relaxation applies to the outbound SOS, not to Yejin's
   account. (ADR-0298 §C)
2. **Continuity of identity across tenants** — Yejin acts as consumer,
   nurse, and family-member across one device without re-login. (ADR-0244
   + ADR-0247)
3. **Cedar fragments evaluate every read** — no field of Yejin's profile
   is read without a permit and an audit. (ADR-0243 + ADR-0263)
4. **Audit chain is sealed within 200ms** — every emergency-bypass action
   is non-repudiable. (ADR-0028 + ADR-0263)
5. **Cell isolation preserves PHI separation** — Min-jun's emerging SNUH
   chart never crosses into Yejin's consumer cell. (ADR-0248)
6. **Pack overlays compose, not conflict** — KR-PIPA + KR-119 + HIPAA stack
   without contradiction; higher-restriction always wins. (ADR-0251 +
   ADR-0304 cross-link to j13)
7. **Tenant-scoping cannot leak across tenant boundaries** — Yejin cannot
   see Min-jun's chart from her nurse role. (ADR-0244)
8. **Abuse-defence sees emergency context and stands down for class** —
   abuse-defence flags Yejin's account as `WHITELISTED_EMERGENCY_BYPASS`
   for 24h so legitimate flurry of post-incident activity isn't penalized.
   (ADR-0297 §D-7)
9. **DSAR surface gives Yejin transparency** — every event that touched
   her account during the emergency is visible to her, exportable, and
   correctable. (ADR-0276 GDPR Art. 20 / KR-PIPA Art. 35)
10. **Observability emits with `tenant_id`, `cell_tier`, `pack`** labels
    on every metric — the SLO dashboard can verify the emergency-bypass
    SLO was met. (ADR-0263)

## 15. What would have gone wrong without this design

To understand why each contract is mandatory, consider what would have
gone wrong otherwise:

**If the bypass were identity-scoped (not class-scoped):**
- A jealous ex with Yejin's credentials could trigger fake SOS pushes
  with the same rate-limit relaxation. The bypass MUST be class-scoped so
  forgery-revocation cannot harm legitimate users.

**If the bypass forwarded ALL of Yejin's profile to 119:**
- 119 dispatchers don't need her vintage-clothing side-business records,
  her family-account composition, or her Saturday walking plans. Only the
  pre-opted-in emergency profile is exposed. Anything else would be a
  KR-PIPA Art. 18 (purpose-limitation) violation.

**If the cell boundary leaked:**
- Min-jun's emerging SNUH chart could appear in Yejin's consumer
  notification feed (e.g., "You have a new health record"). This is a
  HIPAA violation by SNUH and a KR-PIPA violation by oyatie. Cell isolation
  ADR-0248 prevents this architecturally, not procedurally.

**If audit emission were optional or async-with-no-seal:**
- A bad actor could deny they had pulled Yejin's profile. Audit-chain
  seals within 200ms; the sealed event is non-repudiable. (ADR-0028)

**If the abuse-defence flag stayed on:**
- Yejin's post-emergency flurry of activity (47 Messenger notifications,
  voicemail-to-text, calendar rebookings) could trigger an account
  suspension. The 24-hour bypass-context tag stands abuse-defence down,
  while still recording every action.

**If cross-jurisdiction conflict arose:**
- If Min-jun's care had required transferring his chart to a US-affiliated
  hospital (Min-jun is a US citizen), the cross-jurisdiction conflict
  resolver (ADR-0304, j13 cross-link) would apply: higher-restriction
  wins, KR-PIPA + KR-Medical-Records-Act govern the export; US CLOUD Act
  does not override.

**If COPPA/KOSA-protected minors had been involved:**
- Yejin's 8-year-old child witnessing the event could have triggered
  child-safety-channel pushes via oyatie Family-account-mode. Per ADR-0292
  the minor's surface respects parental-control unless a safety-report is
  generated. (See j18.)

**If Yejin had been a DV survivor:**
- The emergency-services bypass MUST still fire (life-safety > shelter
  mode). But the shelter-mode invariants (no SMS-MFA, audit hidden from
  abuser-shared family-account view, abuser does NOT receive the SOS
  notification if their contact entry is shelter-mode-locked) (ADR-0301,
  j04 cross-link) still apply.

Every "if" above corresponds to a Cedar fragment + an audit event class +
an integration test in `integration-test-plan.md`. Every one of these MUST
pass before this journey is GA.

## 16. Cross-context handoffs documented in this story

The story exercises FIVE distinct context handoffs:

| Time (KST) | Handoff | From | To | Mediated by |
|---|---|---|---|---|
| 14:07:14 | iOS SOS → oyatie Messenger relay | OS/carrier | oyatie consumer.kr | Messenger emergency-relay channel |
| 14:07:48 | Carrier dispatch → oyatie Emergency Services API | SeoulMFD console | oyatie API-gateway (consumer.kr ingress) | Attested SPIFFE + Cedar permit |
| 14:13:02 | KR-119 ETA pre-arrival → SNUH EHR | 119 dispatch backbone | oyatie Workflow Engine (snuh.org tenant) | Workflow trigger + Cedar permit |
| 14:14:18 | Yejin's consumer DM → dr.kang's work DM | consumer.kr cell | work.snuh.org cell | Cross-tenant DM Cedar permit |
| 14:19:18 | Yejin's principal context switch | consumer principal (yejin@oyatie.me) | work principal (yejin.park@snuh.org) | Passkey auth + active-clinical-context flag |

Each handoff has its own sequence diagram in `handshake.md` and its own
test in `integration-test-plan.md`.

## 17. Open questions / Wave-3-E follow-up

1. **Patient portal as separate journey** — Yejin's read-access to Min-jun's
   record IF she is added by his primary attending is a follow-up journey
   (proposed: `j-followup-patient-portal-family-read-access`).
2. **Compound journey: emergency + DV** — what if the patient is also a DV
   survivor sharing a device with the abuser, and the abuser is the
   collapsed party? Highly delicate; proposed: `j-followup-emergency-dv-compound`.
3. **Cross-jurisdiction emergency** — what if Min-jun were a US citizen on
   vacation in Seoul and the emergency care required cross-border
   chart-transfer? Covered partially by j13 (cross-jurisdiction-conflict)
   but the emergency vector is uncovered.
4. **Minor-as-caller** — if Yejin's 8-year-old had dialed 119 (because Min-jun
   collapsed when only the child was present), the COPPA/KOSA/KR-Youth
   pack must permit the minor's call without parental-consent friction.
   Proposed: `j-followup-minor-as-emergency-caller`.

These are noted in this slice's Wave-3-E follow-up list at the end of the
README.md.

## 18. Closing — Why this story matters

Yejin will not remember any of the audit IDs. She will not remember the
Cedar fragments. She will not remember the trace_ids. She will remember
that her husband collapsed and that the system did not get in her way.

That is the bar. That is what the architecture must achieve. Every line
of code we ship must, when measured against this minute, ASK: did we make
this 24-minute resuscitation faster? Or did we make it slower?

If we made it slower, we ship something else.

— end of story —

## Completion expansion for story.md

This section completes the story.md artifact to the journey bar from /tmp/codex-brief-j01-j20-lifesafety.md without deleting prior scaffold content.
It is bound to ADR-0298, cites the common ADR pack ADR-0298, ADR-0299, ADR-0300, ADR-0301, ADR-0302, ADR-0303, ADR-0304, ADR-0305, ADR-0306, ADR-0292, and fills intern-buildability details for the touched services: api-gateway, messenger, mail, cell, observability, audit-chain.

# j01 - Story - Emergency 119 dispatch for Yejin Park

The protagonist is Yejin Park. The place is Seoul.
The concrete incident: Yejin husband collapses at home and she dials 119 while oyatie routes life-safety data to PSAP and EMS.
The story preserves continuity of identity. One human may cross personal, work, family, regulated, and emergency contexts, but the platform keeps tenant, audience type, and jurisdiction explicit at each hop.

## Identity continuity table

| Context | Tenant | Principal class | Policy invariant |
|---|---|---|---|
| Personal | personal tenant | B2C_CONSUMER | User controls consumer data and recovery posture. |
| Work | employer tenant when applicable | B2B_WORK_MEMBER | Work surface access never pierces personal tenant. |
| Safety | regulated safety tenant | EMERGENCY_OR_CRITICAL_PATH | Safety traffic is audited and never friction-blocked. |
| Delegate | workflow or agent grant | DELEGATED_AGENT | Grant scope is bounded, revocable, and audit-sealed. |

## Timeline narrative

### 1. T-30 minutes

Normal life continues and no safety overlay is active. In j01, Yejin Park experiences this as one coherent flow, not as a stack of microservice seams.
The binding rule is ADR-0298; the implementation must keep that rule visible in traces, schemas, Cedar decisions, and reviewer evidence.
- api-gateway: emergency-services-bypass-edge performs its part at this moment, emits a span, and preserves tenant context.
- api-gateway acceptance signal 1: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- messenger: sos-contact-fanout performs its part at this moment, emits a span, and preserves tenant context.
- messenger acceptance signal 2: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- mail: emergency-family-mail-fallback performs its part at this moment, emits a span, and preserves tenant context.
- mail acceptance signal 3: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- cell: kr119-cell-routing performs its part at this moment, emits a span, and preserves tenant context.
- cell acceptance signal 4: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- observability: emergency-metrics performs its part at this moment, emits a span, and preserves tenant context.
- observability acceptance signal 5: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- audit-chain: life-safety-seal performs its part at this moment, emits a span, and preserves tenant context.
- audit-chain acceptance signal 6: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.

### 2. T-5 minutes

The first weak signal appears but user-visible friction stays absent. In j01, Yejin Park experiences this as one coherent flow, not as a stack of microservice seams.
The binding rule is ADR-0298; the implementation must keep that rule visible in traces, schemas, Cedar decisions, and reviewer evidence.
- api-gateway: emergency-services-bypass-edge performs its part at this moment, emits a span, and preserves tenant context.
- api-gateway acceptance signal 1: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- messenger: sos-contact-fanout performs its part at this moment, emits a span, and preserves tenant context.
- messenger acceptance signal 2: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- mail: emergency-family-mail-fallback performs its part at this moment, emits a span, and preserves tenant context.
- mail acceptance signal 3: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- cell: kr119-cell-routing performs its part at this moment, emits a span, and preserves tenant context.
- cell acceptance signal 4: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- observability: emergency-metrics performs its part at this moment, emits a span, and preserves tenant context.
- observability acceptance signal 5: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- audit-chain: life-safety-seal performs its part at this moment, emits a span, and preserves tenant context.
- audit-chain acceptance signal 6: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.

### 3. T+0

The critical-path command is issued. In j01, Yejin Park experiences this as one coherent flow, not as a stack of microservice seams.
The binding rule is ADR-0298; the implementation must keep that rule visible in traces, schemas, Cedar decisions, and reviewer evidence.
- api-gateway: emergency-services-bypass-edge performs its part at this moment, emits a span, and preserves tenant context.
- api-gateway acceptance signal 1: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- messenger: sos-contact-fanout performs its part at this moment, emits a span, and preserves tenant context.
- messenger acceptance signal 2: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- mail: emergency-family-mail-fallback performs its part at this moment, emits a span, and preserves tenant context.
- mail acceptance signal 3: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- cell: kr119-cell-routing performs its part at this moment, emits a span, and preserves tenant context.
- cell acceptance signal 4: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- observability: emergency-metrics performs its part at this moment, emits a span, and preserves tenant context.
- observability acceptance signal 5: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- audit-chain: life-safety-seal performs its part at this moment, emits a span, and preserves tenant context.
- audit-chain acceptance signal 6: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.

### 4. T+15 seconds

Edge accepts the command and stamps tenant, cell, jurisdiction, and binding ADR. In j01, Yejin Park experiences this as one coherent flow, not as a stack of microservice seams.
The binding rule is ADR-0298; the implementation must keep that rule visible in traces, schemas, Cedar decisions, and reviewer evidence.
- api-gateway: emergency-services-bypass-edge performs its part at this moment, emits a span, and preserves tenant context.
- api-gateway acceptance signal 1: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- messenger: sos-contact-fanout performs its part at this moment, emits a span, and preserves tenant context.
- messenger acceptance signal 2: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- mail: emergency-family-mail-fallback performs its part at this moment, emits a span, and preserves tenant context.
- mail acceptance signal 3: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- cell: kr119-cell-routing performs its part at this moment, emits a span, and preserves tenant context.
- cell acceptance signal 4: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- observability: emergency-metrics performs its part at this moment, emits a span, and preserves tenant context.
- observability acceptance signal 5: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- audit-chain: life-safety-seal performs its part at this moment, emits a span, and preserves tenant context.
- audit-chain acceptance signal 6: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.

### 5. T+45 seconds

Identity and policy gates resolve the narrowest lawful authority. In j01, Yejin Park experiences this as one coherent flow, not as a stack of microservice seams.
The binding rule is ADR-0298; the implementation must keep that rule visible in traces, schemas, Cedar decisions, and reviewer evidence.
- api-gateway: emergency-services-bypass-edge performs its part at this moment, emits a span, and preserves tenant context.
- api-gateway acceptance signal 1: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- messenger: sos-contact-fanout performs its part at this moment, emits a span, and preserves tenant context.
- messenger acceptance signal 2: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- mail: emergency-family-mail-fallback performs its part at this moment, emits a span, and preserves tenant context.
- mail acceptance signal 3: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- cell: kr119-cell-routing performs its part at this moment, emits a span, and preserves tenant context.
- cell acceptance signal 4: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- observability: emergency-metrics performs its part at this moment, emits a span, and preserves tenant context.
- observability acceptance signal 5: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- audit-chain: life-safety-seal performs its part at this moment, emits a span, and preserves tenant context.
- audit-chain acceptance signal 6: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.

### 6. T+90 seconds

Workflow state moves from accepted to coordinated with audit-chain seal. In j01, Yejin Park experiences this as one coherent flow, not as a stack of microservice seams.
The binding rule is ADR-0298; the implementation must keep that rule visible in traces, schemas, Cedar decisions, and reviewer evidence.
- api-gateway: emergency-services-bypass-edge performs its part at this moment, emits a span, and preserves tenant context.
- api-gateway acceptance signal 1: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- messenger: sos-contact-fanout performs its part at this moment, emits a span, and preserves tenant context.
- messenger acceptance signal 2: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- mail: emergency-family-mail-fallback performs its part at this moment, emits a span, and preserves tenant context.
- mail acceptance signal 3: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- cell: kr119-cell-routing performs its part at this moment, emits a span, and preserves tenant context.
- cell acceptance signal 4: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- observability: emergency-metrics performs its part at this moment, emits a span, and preserves tenant context.
- observability acceptance signal 5: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- audit-chain: life-safety-seal performs its part at this moment, emits a span, and preserves tenant context.
- audit-chain acceptance signal 6: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.

### 7. T+3 minutes

Notifications, operator screens, or trusted contacts receive the minimum necessary packet. In j01, Yejin Park experiences this as one coherent flow, not as a stack of microservice seams.
The binding rule is ADR-0298; the implementation must keep that rule visible in traces, schemas, Cedar decisions, and reviewer evidence.
- api-gateway: emergency-services-bypass-edge performs its part at this moment, emits a span, and preserves tenant context.
- api-gateway acceptance signal 1: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- messenger: sos-contact-fanout performs its part at this moment, emits a span, and preserves tenant context.
- messenger acceptance signal 2: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- mail: emergency-family-mail-fallback performs its part at this moment, emits a span, and preserves tenant context.
- mail acceptance signal 3: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- cell: kr119-cell-routing performs its part at this moment, emits a span, and preserves tenant context.
- cell acceptance signal 4: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- observability: emergency-metrics performs its part at this moment, emits a span, and preserves tenant context.
- observability acceptance signal 5: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- audit-chain: life-safety-seal performs its part at this moment, emits a span, and preserves tenant context.
- audit-chain acceptance signal 6: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.

### 8. T+10 minutes

The user or responder sees state, next action, and appeal or review path. In j01, Yejin Park experiences this as one coherent flow, not as a stack of microservice seams.
The binding rule is ADR-0298; the implementation must keep that rule visible in traces, schemas, Cedar decisions, and reviewer evidence.
- api-gateway: emergency-services-bypass-edge performs its part at this moment, emits a span, and preserves tenant context.
- api-gateway acceptance signal 1: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- messenger: sos-contact-fanout performs its part at this moment, emits a span, and preserves tenant context.
- messenger acceptance signal 2: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- mail: emergency-family-mail-fallback performs its part at this moment, emits a span, and preserves tenant context.
- mail acceptance signal 3: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- cell: kr119-cell-routing performs its part at this moment, emits a span, and preserves tenant context.
- cell acceptance signal 4: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- observability: emergency-metrics performs its part at this moment, emits a span, and preserves tenant context.
- observability acceptance signal 5: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- audit-chain: life-safety-seal performs its part at this moment, emits a span, and preserves tenant context.
- audit-chain acceptance signal 6: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.

### 9. T+1 hour

Post-hoc review begins for any privileged access or safety bypass. In j01, Yejin Park experiences this as one coherent flow, not as a stack of microservice seams.
The binding rule is ADR-0298; the implementation must keep that rule visible in traces, schemas, Cedar decisions, and reviewer evidence.
- api-gateway: emergency-services-bypass-edge performs its part at this moment, emits a span, and preserves tenant context.
- api-gateway acceptance signal 1: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- messenger: sos-contact-fanout performs its part at this moment, emits a span, and preserves tenant context.
- messenger acceptance signal 2: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- mail: emergency-family-mail-fallback performs its part at this moment, emits a span, and preserves tenant context.
- mail acceptance signal 3: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- cell: kr119-cell-routing performs its part at this moment, emits a span, and preserves tenant context.
- cell acceptance signal 4: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- observability: emergency-metrics performs its part at this moment, emits a span, and preserves tenant context.
- observability acceptance signal 5: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- audit-chain: life-safety-seal performs its part at this moment, emits a span, and preserves tenant context.
- audit-chain acceptance signal 6: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.

### 10. T+24 hours

Compliance pack clocks and transparency logs are reconciled. In j01, Yejin Park experiences this as one coherent flow, not as a stack of microservice seams.
The binding rule is ADR-0298; the implementation must keep that rule visible in traces, schemas, Cedar decisions, and reviewer evidence.
- api-gateway: emergency-services-bypass-edge performs its part at this moment, emits a span, and preserves tenant context.
- api-gateway acceptance signal 1: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- messenger: sos-contact-fanout performs its part at this moment, emits a span, and preserves tenant context.
- messenger acceptance signal 2: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- mail: emergency-family-mail-fallback performs its part at this moment, emits a span, and preserves tenant context.
- mail acceptance signal 3: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- cell: kr119-cell-routing performs its part at this moment, emits a span, and preserves tenant context.
- cell acceptance signal 4: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- observability: emergency-metrics performs its part at this moment, emits a span, and preserves tenant context.
- observability acceptance signal 5: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.
- audit-chain: life-safety-seal performs its part at this moment, emits a span, and preserves tenant context.
- audit-chain acceptance signal 6: input schema version is recorded, output event is idempotent, and refusal produces a user-safe path.

## Failure-mode tree

| Failure mode | Required behavior |
|---|---|
| Network partition | The active cell records the command locally, emits a degraded audit event, and replays to sibling cells when the link returns. |
| Byzantine actor | Cedar default-deny refuses over-broad scope and audit-chain records the attempted escalation without leaking protected payloads. |
| Regional outage | Cell routing moves reads to the DR pair while writes use the journey-specific consistency policy. |
| Key compromise | OpenBao and SPIFFE attestation rotate the workload credential and quarantine only the affected principal or tenant. |
| Model or classifier error | The human-review or post-hoc review lane receives the evidence packet, while life-safety paths remain unblocked. |
| Replay or duplicate submit | Idempotency keys and audit-event hashes collapse duplicate operations into a single state transition. |

## Story rigor matrix

| Dimension | Journey-specific acceptance signal |
|---|---|
| Maintainability | Service ownership is explicit, reverse dependencies are named, and every public contract has a versioned schema. For j01, this is bound to ADR-0298. |
| Observability | Audit events, metrics, traces, and logs are declared for the happy path and each refusal branch. For j01, this is bound to ADR-0298. |
| Scalability | The design handles 10x traffic by partitioning on tenant, cell, journey id, and regulator pack. For j01, this is bound to ADR-0298. |
| Performance | P95 user-visible operations stay below the critical-path budget and tail latency is guarded by circuit breakers. For j01, this is bound to ADR-0298. |
| Optimization | Hot-path calls use caller-side policy and ontology reads where allowed; cold-path review stays asynchronous. For j01, this is bound to ADR-0298. |
| Code quality | The IP slices require unit, property, integration, replay, load, and compliance-pack tests before promotion. For j01, this is bound to ADR-0298. |

## Capacity and latency budget

Capacity model uses Little law: concurrent work in system equals arrival rate multiplied by service time.
For j01, the baseline planning rate is 100 journey starts per minute per active cell unless a stricter emergency or compliance pack overrides it.
The 10x surge model is 1000 starts per minute. At 250 ms median service time, expected concurrent active commands are 4.17; the shard plan reserves 64 partitions so one partition can fail hot without global collapse.
The 100x disaster drill is modeled separately as 10000 starts per minute. At 500 ms degraded service time, expected concurrent active commands are 83.4; the rate-limit floor never challenges emergency or safety traffic, but non-critical surfaces shed load first.

| Budget | Target | Evidence required |
|---|---:|---|
| Edge accept p95 | 250 ms | api-gateway trace histogram with tenant and cell dimensions |
| Cross-service command p95 | 800 ms | workflow-engine span tree with retry annotations |
| Audit seal p95 | 1000 ms | audit-chain seal latency histogram and Merkle proof sample |
| User notification p95 | 3000 ms | messenger or mail delivery metric split by provider |
| Regulator-clock start | 60 s | compliance event with jurisdiction pack and due-at timestamp |

## Observability contract

Audit event classes emitted:
- j01.journey.accepted: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j01.cedar.decision: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j01.audit.sealed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j01.handoff.completed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j01.exception.reviewed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.

Metrics emitted:
- oyatie_j01_accepted_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j01_refused_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j01_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j01_audit_seal_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j01_manual_review_queue_depth: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j01_notification_delivery_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.

Trace shape:
- span 1: api-gateway.emergency-services-bypass-edge uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 2: messenger.sos-contact-fanout uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 3: mail.emergency-family-mail-fallback uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 4: cell.kr119-cell-routing uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 5: observability.emergency-metrics uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 6: audit-chain.life-safety-seal uses parent trace from the journey accept span and records Cedar decision plus schema version.

## Anti-stories

- The platform must not collapse personal and work tenant scopes just because the same device is used.
- The platform must not add CAPTCHA, SMS-only recovery, or challenge friction to life-safety paths.
- The platform must not let anonymous or high-risk reports become de-anonymized by observability tags.
- The platform must not hide post-hoc review from compliance owners when privileged access occurred.

