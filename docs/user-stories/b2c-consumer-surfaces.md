---
doc_class: UserStoriesCompendium
title: B2C Consumer Surfaces — User Stories Compendium
status: Draft
date: 2026-05-20
owner_team: council-product + axis-design + axis-frontend
audience: intern-readable
related_adrs:
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
related_prds:
  - microservices/mail/PRD.md
  - microservices/messenger/PRD.md
  - microservices/community/PRD.md
  - microservices/calendar/PRD.md
  - microservices/meet/PRD.md
  - microservices/drive/PRD.md
  - microservices/notes/sdk-plan.md
  - microservices/plugin-app-store/PRD.md
related_memory:
  - feedback_workflow_studio_scope
  - feedback_clean_architecture_requirements
  - feedback_quality_performance_scalability_bar
intent: >
  Specify user stories for every consumer-facing surface at a level of detail
  that an intern can use to design and build the corresponding screens, flows,
  and back-end interactions. Every story is grounded in the PRD's functional
  requirements, the substrate-vs-product layering doctrine (ADR-0245), and the
  oyatie-is-a-tenant doctrine (ADR-0242) so that B2C personal users are
  treated as principals under their own personal tenant under the same
  compliance machinery as enterprise tenants.
---

# B2C Consumer Surfaces — User Stories Compendium

## 1. Purpose

This compendium specifies the consumer-facing user stories for every B2C
personal surface oyatie ships. It is written so that an entry-level engineer
(intern) can:

- Pick a story.
- Draw the screen wireframes from the **Step-by-step actions** + **Expected
  behaviors** sections.
- Identify the back-end calls from the **Precondition** + **Expected
  behaviors** sections.
- Know which edge and error cases the implementation must cover from the
  **Edge cases** + **Error cases** sections.
- Verify the implementation against the **Success outcome** and **Failure
  recovery** statements.

The stories are intentionally redundant where redundancy improves clarity for
new engineers. Where a story references a back-end primitive (e.g.,
"audit-chain emits a `MessageReceived` record"), the underlying contract lives
in the linked PRD; this document does not redefine those contracts.

### 1.1 Doctrinal grounding

Per ADR-0242, every B2C personal user is a **principal under a personal-tier
tenant**. There is no special "B2C bypass" path. The same Cedar gate, the
same audit chain, the same DSAR cascade, the same encryption requirement
applies to every personal user as to every enterprise tenant. The visible
*product surface* differs (Personal pillar UX is intentionally simpler than
Professional pillar UX), but the *substrate machinery* is identical.

Per ADR-0245, every consumer-facing screen belongs to a **product-tier**
µservice (Mail, Drive, Calendar, Meet, Messenger, Community, Notes,
Plugin App Store, Marketplace). Each product calls one or more **substrate**
µservices (Identity, Tenancy, Policy Engine, Ontology, Intelligence,
Audit Chain, Workflow Engine, Comms Email, Consent Graph) per the dependency
direction in ADR-0245 §D-4. Interns building these surfaces never call
substrates directly from the client — every client call goes through the
product's API surface, and the product calls the substrates on the user's
behalf.

### 1.2 How to read a story

Every story below follows this structure:

```
Story ID: <surface>-<persona>-<short-name>
Persona: <which canonical persona acts>
Context: <one-sentence framing>
Precondition: <account state + prior actions>
Step-by-step actions: <numbered; specific gestures>
Expected behaviors: <numbered to match actions>
Edge cases: <3-5 things that can vary>
Error cases: <3-5 things that can break>
Success outcome: <what "done" looks like to the user>
Failure recovery: <what the user can do when something breaks>
```

A "step" is one user gesture (tap, click, type, voice command, key combo).
An "expected behavior" is one observable system response (UI update, API
call, audit event, notification).

### 1.3 Cross-cutting principles (apply to every story)

- **100ms response budget.** Any UI gesture that does not change persistent
  state MUST produce a visible UI response within 100ms. Long operations
  (uploads, AI summaries, video joins) MUST show progress within 100ms.
- **Optimistic UI.** Mutations apply locally first, sync to server in the
  background, and reconcile on response. The user never waits on the network
  for visual feedback unless the action is irreversible.
- **Undo for destructive operations.** Delete, archive, send, leave-channel,
  unsubscribe — all reversible within at least 10 seconds via a visible undo
  affordance.
- **No dark patterns.** Unsubscribe is one tap. Leave-community is one tap.
  Delete-account is one tap (then a confirmation). Cancel-subscription is one
  tap.
- **Accessibility by default.** Every screen passes WCAG 2.2 AA. Every
  gesture has a keyboard equivalent. Every image has alt-text. Every video
  has captions. Every transition respects `prefers-reduced-motion`.
- **Offline-first where applicable.** Notes, Drive, Mail, Calendar, Messenger
  read paths work offline. Writes queue and reconcile on reconnect with
  deterministic conflict resolution (Loro CRDT for Notes; per-tenant CRDT
  for Messenger drafts; last-writer-wins for Calendar with conflict alert).
- **Localization day-one.** Korean, Japanese, Simplified Chinese, Traditional
  Chinese, English (US, UK), Spanish (ES, LATAM), Portuguese (BR, PT),
  German, French, Italian, Dutch, Russian, Arabic (RTL), Hebrew (RTL),
  Hindi, Indonesian, Vietnamese, Turkish — minimum 18 locales at GA.
- **Personal pillar isolation.** Per ADR-0238 + Mail/Messenger/Calendar/Drive
  PRDs, Personal context data NEVER crosses into Professional context. Org
  admins cannot export Personal mailboxes. Personal DMs cannot become
  Professional replies. This is enforced at the kernel layer (not the UI),
  but the UI must communicate the boundary clearly via persona indicators.

---

## 2. Personas

The five canonical personas span age, geography, tech-fluency, and platform
loyalty. Every story uses exactly one persona to keep the example concrete;
the patterns generalize.

### 2.1 Alice — Korean graphic designer, 28, Seoul

- **Devices:** iPhone 15 Pro (primary), 14-inch M3 MacBook Pro (work), iPad
  Pro 11" (sketching). Mostly mobile; switches to laptop for client work.
- **Platforms she uses today:** KakaoTalk (primary messenger), Gmail
  (personal + freelance), Naver Mail (Korean clients), Instagram (work
  portfolio + personal), Figma, Notion, Apple Notes, Apple Calendar.
- **Tech fluency:** High. Knows the difference between cloud and local
  storage. Uses two-factor auth on everything. Has a password manager.
- **Languages:** Korean (native), English (fluent professional). Reads UI in
  Korean by preference; switches to English for design tools.
- **Pain points:** Korean clients expect KakaoTalk-style sticker rituals;
  international clients expect WhatsApp/Signal/email. Currently runs four
  messenger apps.
- **Accessibility:** None. Light photophobia — uses Dark Mode on phone after
  sunset.

### 2.2 Bob — American finance manager, 45, NYC

- **Devices:** iPhone 14 (personal), Dell XPS 13 (work, Windows 11), iPad
  9th-gen (couch use). Email is his life.
- **Platforms he uses today:** Gmail (personal + secondary work), Outlook
  (primary work via Exchange), iMessage (family), Facebook (extended
  family + alumni), WhatsApp (one Argentine cousin), LinkedIn.
- **Tech fluency:** Moderate. Can recover his Apple ID; cannot explain DNS.
  Trusts brand-name products; suspicious of unbranded.
- **Languages:** English (native).
- **Pain points:** Inbox overload (1200/day at peak). Calendar conflicts
  across personal + family + work. Spam phishing his elderly parents.
- **Accessibility:** Reading glasses. Sometimes increases system font to
  ~110%.

### 2.3 Carol — German college student, 18, Berlin

- **Devices:** Pixel 8a (primary), Steam Deck (gaming), Framework laptop
  (Linux, dual-boot to Windows for some classes). Discord is her social
  graph.
- **Platforms she uses today:** Discord (primary, 40+ servers), Telegram
  (study groups + crypto chats), Snapchat (close friends), Spotify, Reddit
  (eight subreddits she actively posts in), Lemmy (Reddit alternative for
  some communities), Matrix (privacy-curious).
- **Tech fluency:** Very high. Self-hosts a Nextcloud instance on a
  Raspberry Pi 5. Knows what self-custody means.
- **Languages:** German (native), English (fluent), some French (school).
- **Pain points:** Surveillance creep. Wants federated alternatives. Active
  in privacy advocacy.
- **Accessibility:** None known. Strongly prefers keyboard-driven UI; uses
  Vimium and arrow-keys everywhere.

### 2.4 David — Japanese retired teacher, 60, Tokyo

- **Devices:** iPhone 13 (1 year old; gift from son), iPad mini 6, occasional
  iMac (~5 years old). Phone-first; touch-screen comfortable; keyboard less so.
- **Platforms he uses today:** LINE (LIN family/community), native iOS Mail
  (no Gmail), Apple Calendar, FaceTime (for grandchildren in Vancouver), NHK
  News app.
- **Tech fluency:** Low to moderate. Comfortable with Apple Mail + LINE
  rituals; new apps feel hostile. Asks his son when something breaks.
- **Languages:** Japanese (native). Some English (reading; not speaking).
- **Pain points:** Tiny text. Overlapping notifications. Apps that ask too
  many questions before doing what he wants.
- **Accessibility:** Reading glasses + 130%+ system font. Sometimes uses
  VoiceOver to listen to long emails.

### 2.5 Erin — Brazilian freelance journalist, 35, São Paulo

- **Devices:** iPhone 15 (work), Samsung Galaxy A54 (source-burner phone),
  ThinkPad X1 Carbon (deep-work, Ubuntu). Travels frequently across LATAM.
- **Platforms she uses today:** WhatsApp (Brazil + most LATAM sources),
  Gmail (work + most editors), Signal (sensitive sources), Telegram (some
  political sources), Twitter/X (still active), Mastodon, Slack (one
  newsroom).
- **Tech fluency:** High. Uses ProtonMail for some sources; understands PGP
  in principle; uses Signal-disappearing-messages.
- **Languages:** Portuguese (native), Spanish (fluent), English (fluent
  professional).
- **Pain points:** Cross-border data residency (some sources insist on EU
  storage). Account portability when she changes burner phones. Federation
  with Matrix sources.
- **Accessibility:** None known. Uses high-contrast mode when working
  outdoors.

---

## 3. Surface 1 — Messenger (personal)

### Story M-01: Alice sends a photo + caption to Bob (single recipient)

- **Persona:** Alice (28, Seoul).
- **Context:** Alice has finished a logo concept and wants to share a JPG
  with Bob, a US-based client she met at a conference last month.
- **Precondition:**
  1. Alice has a personal oyatie account, signed in on her iPhone with
     passkey + Personal pillar selected.
  2. Bob is in Alice's contacts (resolved via Ontology `Person` lookup, per
     messenger PRD §"Ontology reads").
  3. Both users have agreed to a DM (DM thread exists or is creatable).
- **Step-by-step actions:**
  1. Alice taps the Messenger icon on her iPhone home screen.
  2. Alice taps the search bar at the top of the Chats list.
  3. Alice types "Bob".
  4. Alice taps Bob's name in the search results.
  5. The Bob conversation opens. Alice taps the paperclip icon to the left
     of the text field.
  6. The attachment sheet rises. Alice taps "Photo Library".
  7. The system Photos picker opens. Alice taps the most recent photo
     (logo concept).
  8. Photos picker dismisses; the photo appears as a preview thumbnail
     above the text field.
  9. Alice taps the text field and types "first draft — let me know what
     you think 🎨".
  10. Alice taps the send button (paper-plane icon).
- **Expected behaviors:**
  1. Messenger app launches in <500ms; cold start <1s; Chats list shows
     pinned + recent threads.
  2. Search bar focuses; keyboard rises; cursor visible.
  3. Real-time search returns Bob in <100ms per messenger PRD §Performance.
  4. Thread opens; last 50 messages render from local SQLite cache; cursor
     positions at end.
  5. Attachment sheet animates up in 200ms; respects `prefers-reduced-motion`.
  6. iOS Photos permission prompt appears if not previously granted.
  7. Standard iOS Photos picker per Apple HIG.
  8. Photo uploads to staging bucket in background (per messenger PRD
     `file-attachment` BC); preview rendered locally from EXIF-stripped
     thumbnail.
  9. Soft keyboard with emoji + sticker picker available; emoji 🎨 chosen
     from recent.
  10. `MessagePosted` event emits to `messenger.message.v1`; message appears
      in thread with single grey checkmark (sent). On server ACK, checkmark
      becomes double grey (delivered). When Bob's client reads, double
      checkmark becomes blue (read receipt).
- **Edge cases:**
  - Photo is HEIC (iOS default): server transcodes to JPEG for Bob's older
    Android clients; original HEIC retained.
  - Photo size > 5GB (PRD `file-attachment` ceiling): rejected pre-upload
    with "Photo too large; resize?" prompt.
  - Bob is offline: message queues server-side; delivery checkmark stays
    grey until Bob comes online.
  - Alice loses connectivity mid-upload: upload pauses; resumes on
    reconnect per RFC 7233 + tus 1.0 resumable upload.
  - Bob blocks photo previews (privacy mode): photo arrives as blurred
    thumbnail until tapped (Personal pillar opt-in).
- **Error cases:**
  - Network failure on send: message shown with red exclamation; tap
    retries.
  - Storage quota exceeded: prompt "Free up space in Drive?"
  - Malware scan fails (per `file-attachment` BC OPSWAT/ClamAV): photo
    quarantined; user notified "Could not send — content flagged".
  - Bob has blocked Alice: send appears to succeed locally; server suppresses
    delivery per messenger Cedar `policy/block-list.cedar`.
  - DKIM/SPF irrelevant (not mail) but server-side rate-limit may pause
    after 60 messages/min per messenger PRD §Security.
- **Success outcome:** Bob receives the photo + caption within p99 ≤ 100ms
  intra-region; Alice's UI shows double-blue checkmark when Bob reads it.
- **Failure recovery:** If send failed, Alice taps the red exclamation, sees
  "Retry / Delete draft / Save photo to Drive"; choosing Retry uses cached
  EXIF-stripped JPEG; no re-upload from disk needed.

---

### Story M-02: Bob receives a reaction and reads it

- **Persona:** Bob (45, NYC).
- **Context:** Bob is on the subway. Alice has just sent the logo photo.
- **Precondition:**
  1. Bob has the Messenger app installed and signed in with iCloud Keychain
     passkey.
  2. Bob has push notifications enabled, with "Show Previews → When Unlocked".
  3. Bob's phone has cell signal at the station; LTE only (no Wi-Fi).
- **Step-by-step actions:**
  1. Bob's iPhone displays a banner: "Alice — first draft — let me know
     what you think 🎨" with a photo thumbnail.
  2. Bob long-presses the notification banner. (Or taps; documented as
     long-press for context-menu path.)
  3. Bob taps the 👍 reaction in the quick-reaction strip.
  4. Bob then taps the banner itself to open the conversation.
  5. Bob taps the photo full-screen; pinch-zooms to inspect.
  6. Bob taps the back chevron; conversation visible.
  7. Bob types "love the kerning — can you try one in dark teal?" and sends.
- **Expected behaviors:**
  1. Push notification arrives within p99 ≤ 1s of Alice's send (per messenger
     PRD §Performance + APNs/FCM delivery floor).
  2. Long-press surfaces iOS notification quick-actions menu with thumbs-up,
     heart, "Reply…" options.
  3. 👍 reaction sent without opening the app; banner dismisses; Alice's
     thread updates with the reaction within p99 ≤ 250ms.
  4. App opens to Alice's thread; the photo + caption + Bob's just-sent 👍
     reaction all visible.
  5. Photo viewer opens; zoom respects standard pinch gestures.
  6. Back chevron returns to thread view; scroll position preserved.
  7. Reply sends; double-grey checkmark; Alice's `read-receipt-tracker`
     emits a `MessageRead` event with Bob's last-read-message-id.
- **Edge cases:**
  - Bob's phone is locked + Face ID required: notification shows generic
    "1 new message" until unlock.
  - Bob is on Do Not Disturb: notification suppressed; badge count updates
    silently.
  - Alice's photo was sent self-destructing (5 min): Bob sees countdown
    timer; reaction does not extend the timer.
  - Network drop between reaction send and app open: reaction queues locally;
    syncs on reconnect (presence + read-receipt PRD §Performance allows
    coalescing).
  - Bob reacts before the photo finishes downloading: reaction is on the
    message, not the photo; photo retries.
- **Error cases:**
  - Push notification fails (APNs outage): app icon badge increments on
    next foreground refresh.
  - Photo decode fails: placeholder shown with "Photo could not be
    displayed — Retry" tap target.
  - Read receipt fails to send: PRD allows best-effort 99.9% — Alice's UI
    eventually catches up via WebSocket reconnect handshake.
  - Bob's account locked due to suspicious activity: app opens to sign-in
    screen.
  - Reaction synced twice (idempotency bug): server dedupes by
    `(message_id, user_id, reaction_emoji)`.
- **Success outcome:** Within ~3 seconds of Alice's send, Bob has acknowledged
  with 👍 + sent a reply; Alice's UI shows the reaction and reply.
- **Failure recovery:** If the reaction failed, Bob long-presses the message
  in the thread and re-taps the emoji; idempotency dedupes.

---

### Story M-03: Carol creates an 8-person group chat for Berlin friends

- **Persona:** Carol (18, Berlin).
- **Context:** Carol's friend-group is migrating off Discord-DMs to a more
  privacy-respecting messenger for personal life. Eight close friends.
- **Precondition:**
  1. Carol has a personal oyatie account.
  2. The eight friends are in Carol's contacts (Ontology `Person` resolution).
  3. Carol has a photo on her camera roll suitable for the group avatar.
- **Step-by-step actions:**
  1. Carol opens Messenger and taps the pencil (compose) icon top-right.
  2. Carol taps "New Group".
  3. Carol selects 8 contacts by tapping each (rendered with avatars +
     names).
  4. Carol taps "Next".
  5. Carol enters the group name "Berliner Crew 🌃" in the text field.
  6. Carol taps the avatar placeholder and selects a photo from her camera
     roll.
  7. Carol crops the photo to fit the circular avatar.
  8. Carol taps "Create".
  9. Carol enters an intro message: "hey y'all — proper messenger from now
     on 🇩🇪". Taps send.
- **Expected behaviors:**
  1. Compose surface opens; new options visible.
  2. New Group flow appears; checklist of contacts.
  3. Each tap toggles selection; counter updates ("8 selected"). Maximum
     500 per group per messenger PRD §"Per-cell capacity envelope" (no
     limit hit at 8).
  4. Next button enables once ≥1 contact selected.
  5. Group name max 80 chars; emoji rendering with Unicode 16 support.
  6. Photos picker (system).
  7. Crop tool — square-bounded, but rendered as circle.
  8. `ChannelCreated` event emits; `Channel` object writes to Ontology
     with `context_kind: Personal`; all 8 friends receive a system message
     "Carol added you to Berliner Crew 🌃".
  9. Intro message emits as `MessagePosted` to the new channel; all members
     receive in real time.
- **Edge cases:**
  - One of the 8 friends is on an older OS that doesn't render the 🌃
    emoji: fallback character displayed.
  - Carol exceeds group-name length: input is hard-capped at 80 chars.
  - One friend blocks group invitations from non-mutuals: friend is shown
    in selection but receives an opt-in prompt rather than auto-join.
  - Carol selects a photo with detected faces (privacy heuristic): warning
    "Photo contains faces — share publicly?" Carol confirms.
  - Group name contains a slur (content classifier flag): warning before
    create; Carol may proceed (Personal pillar; community guidelines
    differ).
- **Error cases:**
  - Contact resolution fails for one friend (Ontology timeout): retry-once
    inline; if still fails, that friend is omitted with a notice.
  - Photo upload fails: group is created with default avatar; user can
    retry avatar later.
  - Cedar `channel-create-personal.cedar` denies (rate limit, e.g., 10
    group-creates per day): show "Try again in N hours" with retry-after.
  - Group already exists with same name (not blocked, but warned).
  - One contact has deactivated their account: omitted with notice.
- **Success outcome:** Group exists; all 8 members in `ChannelMember` rows;
  intro message visible; Ontology has the new `Channel` linked to all
  members.
- **Failure recovery:** Carol can edit name/avatar/membership later via
  group-settings sheet.

---

### Story M-04: David receives a 20-minute video call from his grandchild

- **Persona:** David (60, Tokyo).
- **Context:** David's granddaughter Yuki, who lives in Vancouver, calls him
  on Sunday evening (Tokyo morning).
- **Precondition:**
  1. David has the messenger app + meet substrate enabled.
  2. David granted camera + microphone permissions previously.
  3. David is signed in via passkey backed by Apple ID; Personal pillar.
  4. Yuki is in David's contacts.
- **Step-by-step actions:**
  1. David's iPhone vibrates + rings with the incoming-call screen — full
     screen even from lock.
  2. David taps the green "Answer" button.
  3. David's camera + mic activate; he sees Yuki's face full-screen and his
     own face in a small picture-in-picture (PiP).
  4. They talk; David taps the message icon mid-call to type a quick note
     "Wait, your mom too?"
  5. After 20 minutes, David taps the red end-call button.
  6. A post-call card appears: duration 20m 14s, no recording (recording is
     off by default).
- **Expected behaviors:**
  1. Call screen overrides lock-screen (per Apple CallKit integration in
     iOS); ringtone honors silent switch + Do Not Disturb rules per Apple
     HIG.
  2. Answer triggers meet substrate's `participant.join` flow within p99 ≤
     1.5s (per meet PRD §Performance "Participant join").
  3. WebRTC media starts; LiveKit SFU intra-region path used (Yuki on
     pack-us-west, David on pack-jp-east; meet PRD allows cross-pack with
     inter-region SFU mesh and tenant-attestation). Glass-to-glass < 250ms.
  4. In-call chat sidebar appears; messages exchanged via messenger
     substrate (cross-µservice via Workflow event `meet.chat.v1`); messages
     persist post-call in a thread bound to the meet instance.
  5. End-call emits `MeetingEnded`; both clients drop media; PiP closes.
  6. Card includes duration, "Send Yuki a thanks message", "Call again"
     buttons.
- **Edge cases:**
  - David's iPhone is on Wi-Fi calling: handoff to cellular if Wi-Fi drops;
    media continuity preserved via LiveKit ICE restart.
  - Yuki's bandwidth drops: meet adaptive bitrate downgrades video to 480p
    + maintains audio; UI shows a small banner "Yuki has a poor connection".
  - David has Bluetooth hearing aids: audio routes to them via standard iOS
    audio routing.
  - David accidentally tilts phone: video orientation flips smoothly; meet
    sends portrait/landscape track per W3C MediaStreamTrack.
  - David's grandchild is calling from a Matrix-bridged client (per
    messenger PRD's Matrix Federation pin): call appears as a normal
    incoming call; bridge handles the SDP translation.
- **Error cases:**
  - David declines: Yuki sees "David is unavailable" + can leave a message.
  - Call drops mid-call: client auto-retries SFU connection; after 30s
    failure, ends with reason.
  - Camera permission revoked: prompt to re-grant.
  - Microphone permission revoked: call proceeds audio-out only with a
    banner.
  - David has Low Power Mode on: video quality auto-caps at 540p; banner
    informs.
- **Success outcome:** A 20-minute call completes; both endpoints get
  audit-chain emissions of `MeetingStarted` + `MeetingEnded`; no recording
  was made; transcript also not produced (consent gated — KR PIPA Art. 15
  pattern applies, but transcription is off by default per meet PRD §FR-03).
- **Failure recovery:** If David accidentally hangs up, "Call again" button
  on post-call card; or in his Recents he can tap the entry.

---

### Story M-05: Erin sends a 12MB PDF to a confidential source

- **Persona:** Erin (35, São Paulo).
- **Context:** Erin is filing a story; needs to send a 12MB PDF of leaked
  financial records to a source for fact-check.
- **Precondition:**
  1. Erin has selected the source from her contacts; the DM is encrypted
     personal-pillar E2E (Olm/Megolm per messenger PRD §Protocols).
  2. Erin has Drive substrate quota with > 12MB available.
- **Step-by-step actions:**
  1. Erin opens the source's DM thread.
  2. Erin taps the paperclip icon.
  3. Erin selects "Files" → navigates to "Downloads" → selects "fy24
     leak.pdf".
  4. Erin sees an upload progress indicator showing 0/12MB.
  5. Erin types "verify ¶3 on p7 — sources concur?" and sends.
- **Expected behaviors:**
  1. DM opens; lock icon visible indicating E2E.
  2. Standard iOS Files picker.
  3. PDF selected.
  4. Multipart resumable upload (drive PRD `upload` BC) starts; client-side
     encryption applied before upload (E2E for personal-pillar); chunk
     boundaries via FastCDC.
  5. Send message containing the file ref + caption; recipient sees a PDF
     attachment with file name + size + preview thumbnail (first page,
     rasterized in gVisor sandbox per drive PRD).
- **Edge cases:**
  - Source is on a slow connection: PDF lazy-loads on tap, not auto-download.
  - Source uses a Matrix-bridged client: Megolm key share with bridge or
    end-to-end fallback per messenger Matrix federation pin.
  - PDF contains embedded JavaScript (potential exploit): drive's preview
    BC renders rasterized PNG only; original PDF served unmodified to
    recipient.
  - 12MB > inline-attachment threshold: attachment uploaded to drive and
    referenced via signed share-link; messenger payload carries only the
    link + manifest.
  - Erin reaches her Drive quota: prompt "Upgrade quota?" with cancel.
- **Error cases:**
  - Upload fails midway: resumes on reconnect via tus 1.0 + chunk manifest.
  - Virus-scan flags PDF: send blocked; Erin notified with "File flagged —
    appeal" link (per drive PRD `dlp-virus-scan` BC).
  - DLP scan flags PDF (e.g., contains national-ID number): policy-dependent;
    in Personal pillar, DLP defaults to warn-not-block.
  - Source has revoked her DM consent: send queues but does not deliver;
    Erin notified within 5s.
  - Source's client doesn't support Megolm: Megolm group session not
    established; Erin sees "End-to-end encryption unavailable for this
    recipient — send anyway?".
- **Success outcome:** Source receives the PDF; can preview inline; tap to
  download full file; audit-chain records the `FileAttached` event scoped
  to Personal pillar (visible only to Erin + source).
- **Failure recovery:** If send failed, the PDF persists in Erin's Drive
  staging; she can retry from the message draft.

---

### Story M-06: Alice sends a self-destructing message (5-min visibility)

- **Persona:** Alice.
- **Context:** Alice is sharing a not-yet-public client name with a friend
  for context; doesn't want it lingering.
- **Precondition:**
  1. The conversation is Personal pillar.
  2. Self-destruct timers are a Personal-pillar-only feature (per messenger
     PRD §Security).
- **Step-by-step actions:**
  1. Alice opens the conversation.
  2. Alice long-presses the text field.
  3. Context menu shows "Set timer for next message".
  4. Alice selects "5 minutes".
  5. Alice types the message and sends.
- **Expected behaviors:**
  1. Conversation visible.
  2. Long-press menu appears within 100ms.
  3. Timer options: 30s, 1m, 5m, 1h, 1d, off.
  4. Selection persists for the next message only; UI shows ⏱ indicator
     above the field.
  5. Message sends with `expires_at` metadata; recipient sees countdown
     when they view it; message + reactions hard-delete from both clients
     + server after expiry.
- **Edge cases:**
  - Recipient screenshots the message: Alice notified (per ADR-0238 + Apple
    HIG screenshot detection; Personal pillar tells the sender).
  - Recipient is on an older client without timer support: message sent
    with persistent fallback; sender notified pre-send "Recipient may not
    honor timer".
  - Recipient never opens the message: timer starts at send; expires
    server-side regardless.
  - System clock skew on either side: server is source of truth; client
    timer is approximate.
  - Alice cancels send before expiry: ordinary unsend window applies; see
    M-07.
- **Error cases:**
  - Timer set but message still readable post-expiry due to client cache
    bug: ground truth is server; client refresh purges.
  - Recipient is on a federated Matrix client that ignores `expires_at`:
    warning shown to Alice.
  - Audit-chain seal of expiry record fails: retried; eventual consistency
    within 5s.
  - User attempts to set timer to 0s (impossible UI but defensive): server
    rejects.
  - Concurrent send of two timed messages with different timers: each has
    its own timer.
- **Success outcome:** Message exists for 5 minutes after recipient view;
  then gone from both clients and server.
- **Failure recovery:** If timer fails to apply, the recipient sees the
  message as persistent and Alice gets a "Timer not applied" notification
  within 1s.

---

### Story M-07: Bob undoes a sent message within 5 seconds

- **Persona:** Bob.
- **Context:** Bob types a hasty reply, hits send, immediately realizes a
  typo.
- **Precondition:**
  1. Bob's account has the default 5-second undo-send window.
- **Step-by-step actions:**
  1. Bob types "thanks, I'll have to it Friday" (typo for "have to do it").
  2. Bob taps send.
  3. Bob sees a snackbar at the bottom: "Sent. Undo (4s)".
  4. Bob taps "Undo".
  5. The message returns to the draft state; Bob fixes the typo, sends.
- **Expected behaviors:**
  1. Message appears in thread with grey checkmark.
  2. Server holds message in 5-second buffer (not yet fanned out) per
     messenger PRD's edit-window per FR-05 pattern.
  3. Snackbar countdown is visible.
  4. Undo cancels server-side fan-out; recipients never see the message;
     message returns to draft input.
  5. Edited message sent fresh.
- **Edge cases:**
  - Recipient was online and somehow saw the message in-flight (race
    condition; rare): server emits a `MessageDeleted` tombstone to the
    recipient client.
  - Bob taps undo at second 4.99: race condition handled by server clock;
    server is authority.
  - Bob navigates away before undo: snackbar persists; if he returns and
    taps, undo still works within window.
  - Bob has multiple in-flight sends: each has its own undo timer.
  - Window length is admin-configurable for B2B but fixed 5s for B2C
    Personal pillar.
- **Error cases:**
  - Undo fails (server already fanned out): message stays; Bob can delete
    explicitly per FR-05 (with `MessageDeleted` tombstone visible).
  - Network drop during undo: client retries; if window expires during
    retry, message is sent.
  - Undo emits an audit-chain `MessageUndone` record visible only to
    server-side ops (no audit visibility to the recipient).
  - Bob accidentally tries to undo a message he received: UI does not
    expose undo for received messages.
  - Race with recipient typing-indicator: irrelevant; typing indicators
    are separate signals.
- **Success outcome:** Recipient never sees the typo; corrected message is
  the first they see.
- **Failure recovery:** If undo unavailable, "Delete for everyone" within
  60 minutes (edit-window per FR-02 of messenger PRD) leaves a "This
  message was deleted" tombstone.

---

### Story M-08: Carol joins a public channel "Berlin Photography"

- **Persona:** Carol.
- **Context:** Carol discovers a 1,500-member public channel for
  Berlin-based photographers via a friend's invite link.
- **Precondition:**
  1. The channel is public (channel-store ACL includes
     `Permission::"public-read"`).
  2. Carol's account is in good standing.
- **Step-by-step actions:**
  1. Carol taps the invite link her friend pasted.
  2. The Messenger app opens to the channel preview screen.
  3. Carol scrolls through the last 10 messages (read-only preview).
  4. Carol taps "Join Channel".
  5. Carol is added; she sees the full message history (up to retention
     policy).
- **Expected behaviors:**
  1. Universal link handler routes to messenger app per Apple Universal
     Links / Android App Links.
  2. Preview shows channel name, description, rules, member count, recent
     activity sample.
  3. Read-only sample loaded; no read receipt sent.
  4. Cedar `channel-join-public.cedar` evaluated; Carol added.
  5. Full history visible subject to channel retention; Carol's name shows
     in member sidebar.
- **Edge cases:**
  - Channel requires admin approval: join requests; admin notified;
    Carol sees "Pending approval".
  - Channel has slow-mode (1 msg per minute): Carol sees indicator.
  - Channel hits member cap (50k per messenger PRD §"Per-cell capacity
    envelope"): Carol shown "Channel full".
  - Channel is in a different language: messenger UI prompts
    auto-translate option (intelligence substrate).
  - Channel is federated to Matrix `#berlin-photo:matrix.org`: join goes
    through Matrix r0.6.1 join flow; Carol's identity bridged.
- **Error cases:**
  - Invite link expired: friendly "This invite has expired — ask for a
    new one".
  - Carol has been banned from the channel: "You cannot join this
    channel".
  - Channel was deleted: "This channel no longer exists".
  - Carol's account is in restricted state: "Action unavailable — see
    account status".
  - Federation hop fails: retry with backoff; user shown "Joining via
    federation… (this may take a few seconds)".
- **Success outcome:** Carol is a member; can post within rate limits;
  receives notifications per her notification preferences.
- **Failure recovery:** Leave via channel-settings → leave; one tap, no
  dark patterns.

---

### Story M-09: David sends a message via voice ("Hey Siri")

- **Persona:** David.
- **Context:** David's hands are wet from washing rice; wants to send his
  daughter "I'll call you tonight" without touching the phone.
- **Precondition:**
  1. David has Siri enabled with messenger as an Intents-aware app.
  2. David's daughter is in contacts; one prior conversation exists.
- **Step-by-step actions:**
  1. David says "Hey Siri".
  2. Siri activates with chime.
  3. David says, in Japanese, "智子に『今晩電話するね』ってメッセージ送って"
     ("Send Tomoko a message saying 'I'll call you tonight'").
  4. Siri confirms: "Send to 智子: 今晩電話するね. Send?"
  5. David says "はい" (yes).
- **Expected behaviors:**
  1. Siri awake.
  2. Standard system chime.
  3. Speech-to-text in Japanese (Apple on-device) extracts intent +
     content; oyatie SiriKit Intents handler resolves "智子" via Ontology
     Person lookup.
  4. Confirmation TTS readback.
  5. Message sent via messenger substrate; `MessagePosted` event;
     audit-chain records "voice-originated" provenance flag.
- **Edge cases:**
  - Multiple contacts named 智子: Siri asks "Which 智子?"
  - David uses Google Assistant on an Android device: equivalent
    AndroidAppActions Intent path.
  - David's Japanese contains numbers: numerals parsed correctly.
  - Background noise (washing rice): Siri may need a second attempt;
    standard Apple HIG handling.
  - David is in CarPlay: messenger Intents are read-aloud only; reply
    composed by voice; never displays sensitive content on CarPlay
    display without auth.
- **Error cases:**
  - Speech recognition fails: Siri asks David to repeat.
  - Permission to messenger not granted: Siri prompts to grant.
  - Network error during send: message queues locally; Siri tells David
    "I'll send it when you're back online".
  - Wrong contact matched: David can say "No, the other 智子" to
    re-disambiguate.
  - Privacy: voice transcripts processed on-device (per Apple Siri
    privacy contract); not transmitted to oyatie servers.
- **Success outcome:** Message sent without David touching the device; he
  rinses his hands and continues cooking.
- **Failure recovery:** If voice fails entirely, David asks his son later
  to show him the touch path again.

---

### Story M-10: Erin federates a message to a Matrix user (cross-platform)

- **Persona:** Erin.
- **Context:** Erin's source uses Element/Synapse on a self-hosted Matrix
  homeserver. They need to exchange end-to-end-encrypted messages without
  the source switching apps.
- **Precondition:**
  1. Erin's personal pillar tenancy allows Matrix federation (per
     messenger PRD §Protocols — cross-pack routing default-deny but
     per-tenant opt-in allowed).
  2. Source's Matrix ID is `@whistleblower:matrix.privacysource.example`.
- **Step-by-step actions:**
  1. Erin opens "New conversation".
  2. Erin types `@whistleblower:matrix.privacysource.example`.
  3. Messenger detects the Matrix ID format and shows a federated-contact
     badge.
  4. Erin taps the badge; preview shows "via Matrix federation —
     end-to-end-encrypted (Megolm)".
  5. Erin types her first message and sends.
- **Expected behaviors:**
  1. New conversation flow.
  2. Input parser recognises Matrix ID syntax.
  3. UI badge informs Erin of federation; per messenger PRD only opt-in
     Personal-pillar tenants get this UX.
  4. Matrix server-server discovery completes via .well-known + key
     verification; Megolm group session established.
  5. Message ciphertext routed via Matrix r0.1.4 federation hop;
     audit-chain records the federation egress per ADR-0145 invariant 1.
- **Edge cases:**
  - Source's homeserver rejects federation: clear UX explaining; Erin
    can paste the source's HS URL for verification.
  - Source replies with attachments via Matrix `m.file` event: drive
    substrate ingests + previews.
  - Source's homeserver fails verification: warning + manual key
    confirmation required.
  - Source switches devices: Megolm key share completed via Matrix's
    key-backup primitives.
  - Source uses a different Matrix homeserver to reply (rejoins): new
    Megolm session.
- **Error cases:**
  - Federation disabled at pack-eu level (some packs may restrict per
    ADR-0240 sovereign-cloud): user notified.
  - Source's homeserver is on r0.6 not r0.1.4 spec: messenger gracefully
    falls back per Matrix protocol negotiation.
  - Cosign / message integrity fails: message rejected with audit-chain
    emission.
  - Rate-limited federation egress: queued with backoff.
  - Matrix server is throttling: user sees "Recipient's server is slow —
    retrying".
- **Success outcome:** Erin and source exchange end-to-end-encrypted
  messages across two homeservers without leaving their primary apps.
- **Failure recovery:** If federation fails persistently, Erin can fall
  back to email + PGP via the mail surface (see Mail stories).

---

### Story M-11: Alice creates a custom sticker pack with emoji

- **Persona:** Alice (graphic designer; loves stickers).
- **Context:** Alice wants Korean-style stickers for her KakaoTalk-emigrant
  friends.
- **Precondition:**
  1. Alice has 12 PNG files in her Drive labeled `sticker-01.png` …
     `sticker-12.png`.
  2. Personal pillar account has sticker pack creation enabled.
- **Step-by-step actions:**
  1. Alice opens messenger settings → "Stickers and emoji" → "Create
     pack".
  2. Alice enters pack name "Crew vibes" + selects category "Reactions".
  3. Alice taps "Add stickers" → selects 12 PNGs from Drive.
  4. Each sticker is auto-cropped + background-removed (intelligence
     substrate).
  5. Alice previews each, drags to reorder.
  6. Alice taps "Publish to my friends".
- **Expected behaviors:**
  1. Sticker editor opens.
  2. Pack metadata captured.
  3. Drive integration via signed share-link.
  4. Background removal via intelligence substrate's image-segmentation
     pipeline; rejects images < 100x100 px.
  5. Live reorder with drag handles; keyboard-accessible reorder shortcut.
  6. Pack published to her personal sticker namespace; sharable via
     deeplink.
- **Edge cases:**
  - One PNG has copyrighted character: ML classifier warns; Alice may
    proceed (Personal pillar; she's the author or has license).
  - Stickers exceed pack size (50 per pack): UI hard-caps; oldest are
    not added; warning shown.
  - Friend uses a federated Matrix client: stickers are sent as inline
    images, not native stickers.
  - Alice wants animated stickers (APNG/Lottie): supported up to 500KB
    per sticker.
  - Alice wants to monetise sticker pack: post-marketplace-launch
    pathway (see Marketplace stories).
- **Error cases:**
  - Background removal fails for some images: Alice can keep original.
  - Drive quota exceeded: prompt to upgrade.
  - Pack name collides (rare; namespaced per user): forced unique.
  - Stickers contain disallowed content per terms: pack rejected with
    appeal path.
  - Animated sticker exceeds 500KB: rejected with "Reduce frames".
- **Success outcome:** Alice can long-press a message + insert a sticker
  from "Crew vibes" pack.
- **Failure recovery:** Alice can edit / delete pack from settings any
  time.

---

### Story M-12: Bob exports an encrypted backup and restores on a new phone

- **Persona:** Bob (just upgraded to iPhone 16).
- **Context:** Bob wants his messenger history on the new phone with the
  Personal-pillar E2E key intact.
- **Precondition:**
  1. Bob is signed in on his old iPhone 14.
  2. New iPhone 16 powered on + signed into iCloud + same Apple ID.
  3. Personal-pillar messages are E2E-encrypted with a user-held key
     (Olm/Megolm device pairing).
- **Step-by-step actions (on iPhone 14):**
  1. Open messenger → settings → "Backup & restore" → "Create encrypted
     backup".
  2. Tap "Generate recovery key".
  3. Tap "Show recovery key" — sees 24 BIP-39 words.
  4. Confirm two random words from the phrase.
  5. Backup is encrypted client-side with a key derived from the recovery
     phrase + Bob's account key; uploaded to Drive substrate under a
     personal-tenant scoped path.
- **Step-by-step actions (on iPhone 16):**
  6. Install messenger app from App Store.
  7. Sign in with passkey.
  8. App prompts "Restore from backup?" — Bob taps "Restore".
  9. App asks for recovery phrase; Bob enters 24 words.
  10. Backup decrypts; history materialises locally; Megolm device keys
      re-established with Bob's Megolm session backup.
- **Expected behaviors:**
  1-5. Standard backup flow; client-side encryption (the server holds
       ciphertext only; key escrow is off).
  6. Standard install.
  7. Passkey auth.
  8. Restore detected via account → backup-manifest lookup.
  9. Recovery phrase rebuilds the key; decrypt succeeds.
  10. Old + new device co-exist briefly; old device can be deauthorised.
- **Edge cases:**
  - Bob loses the recovery phrase: backup is unrecoverable (E2E by design;
    no escrow per messenger PRD §Open Question 5).
  - Backup interrupted mid-upload: tus 1.0 resumable.
  - Bob restores on a non-Apple device: same flow with cross-platform
    Megolm session backup.
  - Backup contains a Personal-pillar E2E group with 200 members: each
    Megolm session restored individually; lazy materialisation.
  - Bob has 10GB of attachments: restore is incremental — first messages,
    then attachments fetched on demand.
- **Error cases:**
  - Recovery phrase entry wrong: 3-attempt rate-limit; then 1-hour lockout.
  - Backup ciphertext corrupt: backup fails with "Backup integrity
    check failed"; Bob can try an older backup point.
  - Drive quota exhausted (backup too large): user prompted to free space.
  - Network drop during restore: resumes from chunk manifest.
  - Bob restores to a region different from his pack-pinned home cell:
    cross-region replication may be required + governed by ADR-0049.
- **Success outcome:** All history (per retention windows) restored on new
  phone within ~10 minutes for a typical user.
- **Failure recovery:** Bob can sign in on new phone without restore; new
  conversations start fresh; old E2E content unrecoverable without the
  phrase.

---

### Story M-13: Carol blocks a harassing user

- **Persona:** Carol.
- **Context:** A persistent harasser in a public channel is DMing her.
- **Precondition:**
  1. Carol has the harasser's profile open or a DM open.
- **Step-by-step actions:**
  1. Carol opens the DM.
  2. Taps the harasser's name → profile sheet.
  3. Taps "Block" at the bottom.
  4. Confirms with "Block and report?" — selects "Block + report harassment".
  5. Optionally adds a description.
- **Expected behaviors:**
  1. DM open.
  2. Profile sheet shows actions: Mute, Block, Report.
  3. Block confirmation modal.
  4. `block-list.cedar` Cedar fragment created; all future
     messages from the harasser silently dropped on server.
  5. Report sent to moderation queue (community PRD `moderation-queue`
     BC if in a channel context; trust-and-safety queue otherwise).
- **Edge cases:**
  - Harasser is on a Matrix federated server: block applies; federation
    drops their messages at the edge.
  - Harasser has multiple accounts (sockpuppets): pattern-match heuristic
    suggests linking known sockpuppets at moderation level.
  - Carol is in a group with the harasser: she can also leave; block
    removes the harasser's content from her view.
  - Carol later wants to unblock: settings → blocked users → unblock.
  - Carol is under 18: parental controls may auto-block + escalate to
    guardian per per-jurisdiction child safety rules.
- **Error cases:**
  - Block fails server-side: retried; UI shows "Blocking…".
  - Report fails to file: queued; Carol notified.
  - Carol blocks herself by mistake (UI prevents this).
  - Block triggers cascade in shared groups: ensure no UI dead-ends.
  - Harasser receives no notification of block (intentional).
- **Success outcome:** Harasser is silenced from Carol's perspective.
- **Failure recovery:** Unblock is always available in settings; no
  punishment to user for unblock-then-reblock.

---

### Story M-14: David enables larger text + listen-aloud for long messages

- **Persona:** David.
- **Context:** David finds messenger text small.
- **Precondition:**
  1. David has accessibility settings open in iOS.
- **Step-by-step actions:**
  1. David opens iOS Settings → Accessibility → Display & Text Size → Larger
     Text → toggles on + slides to 130%.
  2. Opens messenger.
  3. Long messages render larger.
  4. David long-presses a long message → context menu → "Speak".
  5. iOS reads the message aloud in Japanese.
- **Expected behaviors:**
  1. Standard iOS accessibility.
  2. Messenger respects Dynamic Type and Larger Accessibility Sizes per
     Apple HIG.
  3. Layout reflows; no truncation; no horizontal scrolling.
  4. Standard iOS speech menu.
  5. Japanese voice TTS reads aloud.
- **Edge cases:**
  - David's friend sends a message with emoji: TTS reads the emoji name
    (e.g., 「ハートマーク」).
  - David's friend sends a sticker: TTS describes ("Sticker: heart").
  - Mixed Japanese/English messages: TTS switches voices.
  - David increases text to 200%: messenger maintains usable layout;
    text never overflows.
  - David enables Bold Text: messenger respects.
- **Error cases:**
  - TTS voice not installed: prompt to download.
  - Long message exceeds TTS buffer: TTS chunks naturally at sentences.
  - Sticker alt-text missing: TTS says "Sticker".
  - VoiceOver navigation skips a focusable element: lane catches in
    accessibility CI lane.
  - Layout breaks at 200% on some component: bug; logged.
- **Success outcome:** David reads (or listens to) all messages with
  comfort.
- **Failure recovery:** David always has the system VoiceOver fallback.

---

### Story M-15: Erin schedules a message for source's time zone

- **Persona:** Erin.
- **Context:** Erin in São Paulo; source in Lisbon (4 hours ahead). Erin
  wants the message to arrive at source's 9 AM, not 5 AM.
- **Precondition:**
  1. Erin has the DM open.
- **Step-by-step actions:**
  1. Erin types her message.
  2. Long-presses the send button.
  3. Context menu: "Schedule for…" options + custom.
  4. Erin taps "Custom" → picks 9 AM Lisbon time tomorrow.
  5. Confirms.
- **Expected behaviors:**
  1. Standard typing.
  2. Long-press context menu appears.
  3. Options: in 1h, tomorrow morning (per recipient TZ if known),
     custom.
  4. Time picker shows both Erin's local time + recipient's TZ.
  5. Scheduled; message appears in "Scheduled" folder with countdown;
     audit-chain emits `MessageScheduled`.
- **Edge cases:**
  - Recipient TZ unknown: defaults to Erin's TZ + warning.
  - DST transition occurs between schedule and send: server adjusts.
  - Recipient blocks Erin before send time: send suppressed.
  - Erin cancels schedule: removed from queue.
  - Erin edits a scheduled message: edit applies; send time unchanged
    unless she also changes.
- **Error cases:**
  - Schedule in past: rejected with hint.
  - Schedule too far future (> 1 year): rejected.
  - Server clock skew: server is authoritative.
  - Message scheduled but Erin's account is suspended at send time:
    send fails; notified.
  - Recipient deleted account: send fails gracefully.
- **Success outcome:** Source receives message at exactly 9 AM Lisbon.
- **Failure recovery:** Failed scheduled messages return to drafts with
  reason.

---

### Story M-16: Alice mutes a chatty group for 8 hours

- **Persona:** Alice.
- **Context:** "Berliner Crew" group is active during her sleep; she wants
  silence overnight.
- **Precondition:** She's a member of the group.
- **Step-by-step actions:**
  1. Long-press the group in Chats list.
  2. Tap "Mute".
  3. Options: 1h, 8h, until tomorrow, custom, indefinitely.
  4. Tap "8 hours".
- **Expected behaviors:**
  1. Context menu rises.
  2. Mute options visible.
  3. Options enumerated.
  4. Mute applied; group shows a muted-bell icon; no notifications +
     no badge bumps during the window.
- **Edge cases:**
  - Mute persists across devices via account-scoped preference sync.
  - @mentions can override mute per user preference.
  - Sleep Focus is active: mute may stack with Focus.
  - User wakes early and unmutes: tap to unmute.
  - Mute does not affect message receipt or delivery.
- **Error cases:**
  - Preference sync fails: device-local mute still works; eventual
    consistency.
  - Mute window doesn't auto-clear: server time used.
  - Mute conflicts with notification customisation (per-channel sounds):
    mute wins.
- **Success outcome:** Quiet night; full content available in morning.
- **Failure recovery:** One tap to unmute.

---

### Story M-17: Carol pins an important message in a group

- **Persona:** Carol.
- **Context:** A group is planning an event; Carol wants to pin the
  itinerary message.
- **Precondition:** She has channel permission to pin.
- **Step-by-step actions:**
  1. Long-press the message.
  2. Tap "Pin to channel".
- **Expected behaviors:**
  1. Context menu.
  2. Pinned; banner at top of group; `ChannelMessagePinned` audit event.
- **Edge cases:**
  - Multiple pins; UI shows count + tap to see all.
  - Unpinning by another mod: notification to Carol.
  - Pinned messages search-boosted.
  - Pinned message author leaves channel: pin retained; author noted.
  - Pinned message deleted: pin auto-removed.
- **Error cases:**
  - Permission denied: action greyed out.
  - Pin limit reached (e.g., 10): UI prompts to unpin one.
  - Race condition with concurrent pin: server resolves.
  - Pin syncs slowly to other clients: eventual consistency.
  - Audit fails: ops alert; pin still applied.
- **Success outcome:** Pinned message highlighted for all members.
- **Failure recovery:** Unpin by long-press → "Unpin".

---

### Story M-18: David ports messenger history when changing iPhones with iCloud

- Already covered in M-12 (Bob's flow); David's flow differs only in that
  he asks his son to help. Add as supplementary persona-variant.

### Story M-19: Erin starts an ephemeral one-to-many broadcast channel

- **Persona:** Erin.
- **Context:** Erin runs a "tip line" — sources can DM her anonymously;
  she broadcasts safety alerts back.
- **Precondition:** Personal-pillar account with broadcast channel
  capability (per messenger PRD; subset of channel features).
- **Step-by-step actions:**
  1. Settings → "Create broadcast channel" → name "São Paulo Source
     Safety".
  2. Configure: subscribers anonymous to each other; Erin sees pseudonyms
     only; admin can disable replies.
  3. Publish first message.
- **Expected behaviors:** Channel created; one-to-many publishing; sources
  receive via opt-in subscribe link.
- **Edge cases:** sources unsubscribe one-tap; message edit possible
  within 60 min; pinned for new joiners.
- **Error cases:** subscriber cap (e.g., 10k) reached; broadcast rate
  limits; deletion preserves audit-chain.
- **Success outcome:** Sources receive safety alerts.
- **Failure recovery:** Erin can suspend channel + restore later.

---

### Story M-20: Alice exports her DM with Bob as a portable archive (GDPR Art. 20)

- **Persona:** Alice.
- **Context:** Alice wants a portable archive of her DM with Bob to
  reference offline.
- **Precondition:** Personal-pillar; both parties consent (per Personal
  pillar isolation only Alice's side is exportable; Bob's messages within
  it form part of her record).
- **Step-by-step actions:**
  1. Settings → Privacy → Export data → DM with Bob → Format: JSON +
     attachments.
  2. Enter email for export delivery.
  3. Confirm.
- **Expected behaviors:**
  1. Form opens.
  2. Email captured (or in-app download).
  3. DSAR-like cascade runs (per ADR-0242 §"DSAR cascade"); within 30
     days (GDPR Art. 12) — typically minutes for a single DM; download
     link emailed.
- **Edge cases:** archive size limits (large attachments); separate
  download for > 2GB; export bundle signed (Ed25519) for integrity.
- **Error cases:** export fails; user notified; retry available.
- **Success outcome:** Alice receives a verifiable archive.
- **Failure recovery:** Customer support can re-trigger.

---

## 4. Surface 2 — Mail (personal)

### Story Mail-01: Alice composes a rich-text mail with inline image + 3 attachments + scheduled send

- **Persona:** Alice.
- **Context:** Sending a client deliverable for tomorrow 9 AM.
- **Precondition:**
  1. Alice has a Personal-pillar mailbox at her chosen address (e.g.,
     `alice@oyatie.com` or her custom domain).
  2. Drive substrate has files ready.
- **Step-by-step actions:**
  1. Open Mail → tap pencil (compose).
  2. To: enters client email; CC: studio partner.
  3. Subject: types "Brand kit v3 — review".
  4. Body: types rich text with bold "v3" and italic "minor adjustments".
  5. Taps inline-image icon → selects logo PNG → resizes to 50% width.
  6. Taps paperclip → selects 3 PDFs from Drive (10MB total).
  7. Taps clock icon → "Schedule send" → picks "Tomorrow 9:00 AM".
  8. Reviews preview → taps Send.
- **Expected behaviors:**
  1. Composer opens; full Apple HIG / Material 3 / Fluent 2 idioms.
  2. To/CC autocomplete from Ontology Person.
  3. Subject saved on each keystroke (draft autosave).
  4. WYSIWYG editor; matches mail PRD §FR-03 IMAP/JMAP/REST stack.
  5. Inline image embedded as `cid:` reference per RFC 5322 + MIME.
  6. Attachments uploaded via Drive substrate; share-link or attachment
     mode per size; inline preview thumbs.
  7. Schedule UI shows local + recipient TZ.
  8. Compose closes; mail moves to "Scheduled" folder; send fires at
     scheduled time.
- **Edge cases:** Attachment > 25MB triggers share-link substitution
  automatically (per mail/drive bridge); reschedule possible until 1 min
  before send; mail can be cancelled in Scheduled folder.
- **Error cases:** DKIM key not provisioned (rare, new domain) — send
  fails with "Domain setup incomplete"; recipient bounce returned async;
  draft retained.
- **Success outcome:** Mail delivered at exactly 9 AM with DKIM + SPF +
  DMARC alignment per mail PRD §Security.
- **Failure recovery:** Reschedule, edit, or unschedule the queued mail
  anytime before send.

---

### Story Mail-02: Bob applies a filter "From boss, marks as important"

- **Persona:** Bob.
- **Context:** Bob's boss's mails are missed in flood; Bob makes a filter.
- **Precondition:** Bob signed in; boss's email known.
- **Step-by-step actions:**
  1. Settings → Filters → "Add filter".
  2. Conditions: From = `boss@bigfirm.com`. Optional: subject contains.
  3. Actions: Mark as important; apply label "Boss"; never spam.
  4. Save → "Apply to existing mail?" → Yes → 142 mails relabel.
  5. Test: boss sends a mail; arrives marked important + labelled.
- **Expected behaviors:** Filter UI mirrors Gmail patterns; conditions
  composable; tested in dry-run with count preview before save; audit-chain
  emits `MailFilterCreated`.
- **Edge cases:** Filter could conflict with another (e.g., spam rule);
  precedence shown.
- **Error cases:** Filter syntax error; UI prevents save; backend
  re-validates.
- **Success outcome:** Important boss mail never missed.
- **Failure recovery:** Filter editable anytime; can disable rather than
  delete.

---

### Story Mail-03: Carol sets up sub-addressing for Facebook

- **Persona:** Carol.
- **Context:** Carol wants `carol+facebook@oyatie.com` to land in a folder
  she can purge if FB sells her email.
- **Precondition:** Carol's mailbox supports RFC 5233 sub-addressing
  (mail PRD's IMAP/JMAP/REST contract).
- **Step-by-step actions:**
  1. Settings → "Sub-addresses & aliases" → "Add rule".
  2. Pattern: `+facebook` → Folder: "Facebook" → Auto-archive after 30d.
  3. Save.
  4. Carol uses `carol+facebook@oyatie.com` when signing up for FB.
- **Expected behaviors:** Inbound mail to that address routed
  automatically.
- **Edge cases:** Tracker-blocking enabled (strips pixel + 1x1 images);
  user notified.
- **Error cases:** Wildcard collisions; UI prevents.
- **Success outcome:** Carol can audit + delete an entire "Facebook"
  bucket in one action.
- **Failure recovery:** Rule editable; sub-addresses can be retired.

---

### Story Mail-04: David recovers a deleted mail from Trash

- **Persona:** David.
- **Context:** David deleted a bill mail by mistake.
- **Precondition:** Trash retention is 30 days (mail PRD §Retention).
- **Step-by-step actions:**
  1. Sidebar → Trash.
  2. Search for "電気" (electricity).
  3. Tap the bill mail → 3-dot menu → "Move to Inbox".
- **Expected behaviors:** Standard trash UX; restore puts mail back where
  it was.
- **Edge cases:** Past 30 days: gone unless legal hold or backup.
- **Error cases:** Trash empty; clearly stated.
- **Success outcome:** Mail back in Inbox.
- **Failure recovery:** None needed.

---

### Story Mail-05: Erin migrates 5 years of mail from Gmail via IMAP

- **Persona:** Erin.
- **Context:** Erin moves from Gmail to oyatie mail.
- **Precondition:** Gmail account exists; she has an App Password.
- **Step-by-step actions:**
  1. Settings → Import → "From Gmail (IMAP)".
  2. Enter source email + App Password.
  3. Choose folders to import + retention class for each.
  4. Start import.
- **Expected behaviors:** Per mail PRD §FR-08, import preserves source
  hash + folder labels + retention class; progress meter; per-100-mail
  status update; ETA visible.
- **Edge cases:** Mail with non-UTF8 headers; mail PRD's hardened RFC
  5321/5322 parser handles; problematic mails quarantined for review.
- **Error cases:** Gmail rate-limits; throttles automatically; resumes.
- **Success outcome:** Five years of mail in Erin's new mailbox with
  intact chain-of-custody.
- **Failure recovery:** Pause/resume; partial imports recoverable.

---

### Story Mail-06: Alice uses Smart Compose (opt-in) to write a reply

- **Persona:** Alice.
- **Context:** Long client reply; she enables AI assist.
- **Precondition:** Personal-pillar opt-in to Intelligence substrate per
  ADR-0220/0255 + ADR-0242 (Alice is a principal under her personal
  tenant; she gives consent at the API boundary; no data crosses tenants).
- **Step-by-step actions:**
  1. Reply → "Compose with assist".
  2. Provide a one-sentence intent + tone.
  3. AI drafts; Alice reviews + edits inline.
  4. Sends.
- **Expected behaviors:** Intelligence substrate provides streaming
  completion; Alice can stop / regenerate / accept partials; provenance
  watermark in audit chain.
- **Edge cases:** AI-Act low/medium risk classification visible;
  Personal-pillar disclosure of "AI-assisted".
- **Error cases:** AI service slow or unavailable; UI degrades gracefully
  to manual.
- **Success outcome:** Mail composed faster; quality matches Alice's
  voice (she edited).
- **Failure recovery:** Always editable; AI can be disabled in settings.

---

### Story Mail-07: Bob blocks a spammer + reports

- **Persona:** Bob.
- **Context:** Persistent phishing emails.
- **Precondition:** Mail open.
- **Step-by-step actions:**
  1. Open the mail.
  2. 3-dot menu → "Block sender + report phishing".
  3. Confirm.
- **Expected behaviors:** Sender added to per-mailbox blocklist; report
  routed to anti-spam (per mail PRD §FR-10 DLP/abuse).
- **Edge cases:** Sender spoofs different domains; abuse classifier
  groups them.
- **Error cases:** False positives: undo via "Block sender" history.
- **Success outcome:** Cleaner inbox.
- **Failure recovery:** Unblock via settings.

---

### Story Mail-08: Carol shares a draft with co-author via Drive integration

- **Persona:** Carol.
- **Context:** Carol drafts a manifesto for her community; co-edits with
  a friend.
- **Precondition:** Drive integration enabled.
- **Step-by-step actions:**
  1. Compose mail → menu → "Save as collaborative draft".
  2. Add co-author by email.
  3. Co-author joins via link; both edit; comments inline.
  4. Carol sends final.
- **Expected behaviors:** Draft stored in Drive as a `.eml`-equivalent +
  rich-text doc; concurrent edits via Loro CRDT (notes-style); audit-chain
  records co-authoring.
- **Edge cases:** Conflict resolution; CRDT merges.
- **Error cases:** Co-author lacks permission: prompted.
- **Success outcome:** Polished collaborative mail.
- **Failure recovery:** Draft history versioned.

---

### Story Mail-09: David sets vacation responder + delegation

- **Persona:** David (on cruise).
- **Context:** David hands off mail to his son for a week.
- **Precondition:** Delegation enabled.
- **Step-by-step actions:**
  1. Settings → "Out of office" → set 2026-06-10 → 2026-06-17.
  2. Set message in Japanese + English.
  3. Settings → "Delegate access" → enter son's email → access "read +
     reply on my behalf".
- **Expected behaviors:** Responder auto-replies (rate-limited per
  recipient); delegation gates per Cedar `mail-delegate.cedar`;
  audit-chain emits per-delegate-action.
- **Edge cases:** Recursive auto-reply loop avoided per RFC 3834.
- **Error cases:** Delegation revoke any time.
- **Success outcome:** David's mail is handled; on return, full audit log.
- **Failure recovery:** Disable delegation anytime; previous delegate
  actions retained for audit.

---

### Story Mail-10: Erin replies-all to a newsletter only to the author (not list)

- **Persona:** Erin.
- **Context:** Industry newsletter; she wants to respond to author only.
- **Precondition:** Mail open.
- **Step-by-step actions:**
  1. Tap "Reply" (not Reply-All).
  2. UI shows recipients clearly; only author.
  3. Compose + send.
- **Expected behaviors:** "Reply" vs "Reply All" visually distinct; smart
  warning if "Reply All" would hit > 50 people ("Reply to all 412
  people?").
- **Edge cases:** Mailing list headers (RFC 2369 List-Reply): mail honors
  list semantics with clear UX.
- **Error cases:** Accidental reply-all undoable within 5s send-undo
  window.
- **Success outcome:** Only the author receives; the list does not.
- **Failure recovery:** Send-undo within 5s; otherwise a follow-up
  apology mail.

---

### Story Mail-11: Carol enables 2FA via passkey

- **Persona:** Carol.
- **Context:** Carol moves from TOTP to passkey.
- **Precondition:** Account exists.
- **Step-by-step actions:** Settings → Security → "Set up passkey" → biometric.
- **Expected behaviors:** WebAuthn registration; passkey synced to
  device's authenticator (Apple Passwords, Google Password Manager,
  Bitwarden).
- **Edge cases:** Multiple devices; passkeys synced via platform.
- **Error cases:** Authenticator unavailable: TOTP fallback retained.
- **Success outcome:** Phishing-resistant sign-in.
- **Failure recovery:** Recovery via recovery key.

---

### Story Mail-12: Bob searches "from:boss subject:Q4 budget" with mobile keyboard

- **Persona:** Bob.
- **Context:** Inbox search.
- **Precondition:** Mailbox indexed.
- **Step-by-step actions:** Mail → search field → types the query.
- **Expected behaviors:** Search p99 ≤ 500ms on 100k-message mailbox (per
  mail PRD §Performance); encrypted token search returns Cedar-permitted
  results.
- **Edge cases:** Query operators (`from:`, `to:`, `has:attachment`,
  `before:`, `older_than:`) supported.
- **Error cases:** Query syntax error: helpful inline tip.
- **Success outcome:** Right mail in seconds.
- **Failure recovery:** Refine query; clear filters.

---

### Story Mail-13: Alice opens an encrypted (S/MIME) mail from a designer collective

- **Persona:** Alice.
- **Context:** Collective sends S/MIME-signed + encrypted.
- **Precondition:** Alice has S/MIME cert installed.
- **Step-by-step actions:** Tap mail → see green lock icon "End-to-end
  encrypted + signature valid".
- **Expected behaviors:** Per mail PRD §FR-14 S/MIME / PGP signature
  rendered with verification status.
- **Edge cases:** Cert expired: warning; mail still readable but flagged.
- **Error cases:** Decryption fails: cert missing message + import path.
- **Success outcome:** Mail decoded; provenance verified.
- **Failure recovery:** Import cert; retry decrypt.

---

### Story Mail-14: David receives a phishing-flagged mail; learns to recognise

- **Persona:** David.
- **Context:** Phishing alert from anti-spam.
- **Precondition:** Mail PRD's abuse path active.
- **Step-by-step actions:** Mail arrives with red banner "Possible
  phishing — do not click links".
- **Expected behaviors:** Banner explains why (link mismatch, sender
  spoofed); "What is phishing?" educational link.
- **Edge cases:** Genuine but flagged: "Mark as safe" path with
  confirmation.
- **Error cases:** False alarm: retrievable.
- **Success outcome:** David avoids the phish.
- **Failure recovery:** Sandbox open via "View safely" rendering
  scripts-disabled.

---

### Story Mail-15: Erin uses sub-addressing per source to compartmentalise

- **Persona:** Erin (extensions of Carol's Mail-03 pattern).

### Story Mail-16: Carol unsubscribes from a newsletter with one tap

- **Persona:** Carol.
- **Context:** Anti-dark-pattern UX.
- **Precondition:** Newsletter with `List-Unsubscribe` header (RFC 8058).
- **Step-by-step actions:** Mail open → top banner "Unsubscribe" → tap → done.
- **Expected behaviors:** One-tap unsubscribe; never opens browser
  unnecessarily; confirmation toast.
- **Edge cases:** Sender doesn't honor unsubscribe: server-side filter
  added automatically.
- **Error cases:** Unsubscribe HTTP returns 5xx: server-side filter
  fallback.
- **Success outcome:** No more from that sender.
- **Failure recovery:** Block + report if persists.

---

### Story Mail-17: David enables 200% zoom + listens to a mail via VoiceOver

- **Persona:** David.
- See M-14 pattern; mail-specific: VoiceOver reads sender, subject,
  date, then body; Skim mode.

### Story Mail-18: Alice marks a mail as Personal pillar by accident; pillar guard catches

- **Persona:** Alice.
- **Context:** Dual-context isolation per mail PRD §Tenant Outcome 2.
- **Precondition:** Alice has both pillars; the mail is in Personal.
- **Step-by-step actions:** Alice tries to forward Personal mail to a
  Professional client.
- **Expected behaviors:** Persona indicator warning: "This is a Personal
  pillar mail; forwarding to Professional context requires confirmation +
  audit"; Cedar `dual-context-cross-boundary` gate evaluated.
- **Edge cases:** Forward allowed but with explicit consent + audit record.
- **Error cases:** Cross-pillar leak denied entirely if policy strict.
- **Success outcome:** Alice avoids accidental mix.
- **Failure recovery:** Audit reviewable.

---

### Story Mail-19: Bob delegates an admin assistant role on his pro mailbox

- Out of B2C personal scope; noted for cross-pillar context.

### Story Mail-20: Erin requests a GDPR/PIPA export of her entire mailbox

- See DSAR cascade in ADR-0242 + mail PRD §FR-08 export.
- **Step-by-step actions:** Settings → Privacy → Export mailbox.
- **Expected behaviors:** Export bundle within 30-day SLA; chain-of-
  custody seal.
- **Failure recovery:** Customer support can re-trigger.

---

## 5. Surface 3 — Community (personal)

### Story C-01: Carol joins a public Reddit-equivalent "Berlin Photography" community

- **Persona:** Carol.
- **Context:** Carol joins via discovery search.
- **Precondition:** Personal-pillar account.
- **Step-by-step actions:**
  1. Community surface → search "Berlin Photography".
  2. Top result: 12k members.
  3. Carol scrolls feed read-only.
  4. Taps "Join".
  5. Permissions modal: "Receive notifications? Yes/Customize/No".
- **Expected behaviors:** Per community PRD §FR-01, joinable; feed render
  p99 ≤ 300ms.
- **Edge cases:** Approval required for some communities; rules acceptance
  modal.
- **Error cases:** Banned; rate-limited; community full.
- **Success outcome:** Carol is in.
- **Failure recovery:** Leave one-tap.

---

### Story C-02: Bob asks a Q&A question + accepts best answer

- **Persona:** Bob.
- **Context:** Bob asks a question in a finance Q&A community.
- **Precondition:** Bob is a member.
- **Step-by-step actions:**
  1. Community → "Ask a question" button.
  2. Title, body, tags (e.g., taxes, FY24), choose "Q&A mode".
  3. Submit.
  4. Receives 5 answers in 2 days.
  5. Selects best → "Accept answer".
- **Expected behaviors:** Per community PRD §FR-02, voted Q&A; accepted-
  answer surfaced at top; audit-chain emits `AnswerAccepted`.
- **Edge cases:** Bob un-accepts to accept a better one later.
- **Error cases:** Self-acceptance allowed but flagged.
- **Success outcome:** Future searchers see the accepted answer first.
- **Failure recovery:** Edit / re-tag / re-accept anytime.

---

### Story C-03: Alice creates her own "graphic designers" community + sets moderation rules

- **Persona:** Alice.
- **Context:** Niche community for Korean-English bilingual designers.
- **Precondition:** Account in good standing.
- **Step-by-step actions:**
  1. Communities → "Create" → name, slug, description, languages,
     visibility (public/restricted/private).
  2. Pick rules from templates + custom (no NSFW, attribution required).
  3. Enable Q&A + KB tabs.
  4. Invite first 10 friends.
- **Expected behaviors:** Per community PRD §FR-06 moderator actions;
  Ontology writes; audit chain.
- **Edge cases:** Slug collision; namespacing per user.
- **Error cases:** Moderator role assignment fails (Cedar policy);
  retried.
- **Success outcome:** New community exists; Alice is owner-moderator.
- **Failure recovery:** Edit any setting; community deletable with grace
  period.

---

### Story C-04: David browses KB articles for his neighborhood association

- **Persona:** David.
- **Context:** Neighborhood community runs as a community on oyatie.
- **Step-by-step actions:** Open → KB tab → browse categories → tap
  article.
- **Expected behaviors:** Per community PRD `kb-article-store` BC;
  reading mode; print/share/save.
- **Edge cases:** Article revisions visible; multilingual articles.
- **Error cases:** Article moved or deleted; redirect.
- **Success outcome:** Reads policy on garbage day.
- **Failure recovery:** Search alternative.

---

### Story C-05: Erin participates in an AMA with an industry expert

- **Persona:** Erin.
- **Context:** AMA event in an investigative-journalism community.
- **Step-by-step actions:** Event banner → "Join AMA" → asks a question →
  upvotes others.
- **Expected behaviors:** Event-mode UI; ranked Q&A; live-update.
- **Edge cases:** Q&A volume; moderator deletes off-topic.
- **Error cases:** Expert leaves early; UI updates.
- **Success outcome:** Erin's question answered.
- **Failure recovery:** AMA transcript available afterward.

---

### Story C-06: Alice federates her community to Lemmy/ActivityPub

- **Persona:** Alice.
- **Context:** Wants to reach Lemmy + Mastodon audiences.
- **Precondition:** Federation feature enabled per per-pack overlay.
- **Step-by-step actions:** Community settings → "Federation" → "Enable
  ActivityPub" → confirm.
- **Expected behaviors:** Outbound ActivityPub stream; inbound from
  followers; mod tools across federation.
- **Edge cases:** Defederation by remote instance; per-instance
  blocklists.
- **Error cases:** Federation failure: per-message retry.
- **Success outcome:** Cross-platform reach.
- **Failure recovery:** Disable federation any time.

---

### Story C-07: Carol moderates with automod rules

- **Persona:** Carol (moderator of a community she joined later).
- **Step-by-step actions:** Mod tools → "Add automod rule" → triggers
  (regex; user age; karma) → action (remove; quarantine; report).
- **Expected behaviors:** Per community PRD `moderation-queue` BC; rules
  composable.
- **Edge cases:** False positives; appeals workflow.
- **Error cases:** Regex pathology; safe interpreter.
- **Success outcome:** Clean community.
- **Failure recovery:** Rule editable; appeals visible.

---

### Story C-08: Bob reports a spam post + sees outcome via appeals workflow

- **Persona:** Bob.
- **Step-by-step actions:** Post → 3-dot → "Report" → reason → submit.
- **Expected behaviors:** Per community PRD §"Audit + Compliance" and
  Section-230 stance; queue entry; eventual mod verdict; appeal path for
  poster.
- **Edge cases:** False flag: no penalty.
- **Error cases:** Mod queue backlog: estimated SLA shown.
- **Success outcome:** Post removed if violating.
- **Failure recovery:** Appeals reviewable.

---

### Story C-09: David participates in a discussion forum thread

- Standard threaded reply via community PRD `thread-tree`.

### Story C-10: Alice publishes a KB article with images + revisions

- Per `kb-article-store`; revisions sealed.

### Story C-11: Carol subscribes to tag "Berlin events"

- Per community PRD §FR-08; notifications + email digest configurable.

### Story C-12: Erin pins an investigative tipline post

- Mod action; community member visibility.

---

## 6. Surface 4 — Calendar (personal)

### Story Cal-01: Alice creates a yearly birthday reminder for Mom

- **Persona:** Alice.
- **Step-by-step actions:**
  1. Calendar → "+ Event".
  2. Title "Mom's birthday"; date 1989-03-15; all-day; yearly RRULE.
  3. Add reminder 7 days before + 1 day before.
- **Expected behaviors:** Per calendar PRD `recurrence-engine`, RFC 5545
  RRULE applied; reminders queued.
- **Edge cases:** Leap-year birthdays; per Apple HIG, calendar treats
  Feb 29 events with policy choice.
- **Error cases:** TZ mismatch on travel.
- **Success outcome:** Annual reminders fire.
- **Failure recovery:** Edit RRULE anytime.

---

### Story Cal-02: Bob accepts a Gmail invitation (interop)

- **Persona:** Bob.
- **Precondition:** External RFC 5546 invitation lands in mail.
- **Step-by-step actions:** Open mail with invite → "Accept" inline →
  added to calendar.
- **Expected behaviors:** Calendar PRD §FR-06 RSVP flow.
- **Edge cases:** Tentative; Maybe; counter-propose.
- **Error cases:** Calendar conflict warning; offered alternative slots.
- **Success outcome:** Event in calendar.
- **Failure recovery:** Decline later; sends update.

---

### Story Cal-03: Carol syncs to native iOS Calendar via CalDAV

- **Persona:** Carol.
- **Precondition:** iOS Settings → Accounts → CalDAV account configured.
- **Step-by-step actions:** Enter `cal.oyatie.dev` + Carol's credentials.
- **Expected behaviors:** Per calendar PRD §FR-09; RFC 4791 CalDAV
  read/write.
- **Edge cases:** Two-way sync; conflict resolution.
- **Error cases:** mTLS handshake fails; clear UX.
- **Success outcome:** All events in Apple Calendar.
- **Failure recovery:** Re-add account.

---

### Story Cal-04: David sets "do not disturb" evenings

- **Step-by-step actions:** Settings → Quiet hours → daily 21:00-07:00.
- **Expected behaviors:** Reminders suppressed; events without alarms
  unaffected.
- **Edge cases:** Travel TZ; quiet hours follow body clock or device
  clock — user choice.
- **Success outcome:** Restful nights.
- **Failure recovery:** Toggle off.

---

### Story Cal-05: Erin shares her calendar with husband (read-only)

- **Step-by-step actions:** Calendar → settings → "Share" → husband's
  email + permission "Free/busy only" (or "Full event details").
- **Expected behaviors:** Husband sees as shared in his app; Ed25519
  audit-chain emits `CalendarShared`.
- **Edge cases:** Sharing across personal-pillar boundaries — allowed
  within personal tenant.
- **Error cases:** Permission revocation propagates.
- **Success outcome:** Coordinated.
- **Failure recovery:** Unshare anytime.

---

### Story Cal-06: Alice creates a recurring weekly studio block

- RRULE weekly; per calendar PRD §FR-02.

### Story Cal-07: Carol exports calendar to .ics for backup

- Per FR-08.

### Story Cal-08: David sees DST transitions handled

- Per FR-11 IANA tz; ICU.

### Story Cal-09: Bob proposes a counter time

- Per FR-06.

### Story Cal-10: Erin imports a .ics from a conference

- Per FR-07; 10k events parsed.

### Story Cal-11: Alice deletes a single instance of a recurring event

- Per RFC 5545 EXDATE.

### Story Cal-12: Bob creates a private "Hidden from family" calendar

- Personal-pillar sub-calendar; visibility scoped.

---

## 7. Surface 5 — Meet (personal)

### Story Meet-01: Alice 1:1 video call with sister in Vancouver via mail link

- **Persona:** Alice.
- **Step-by-step actions:** Mail with meet link → tap → join.
- **Expected behaviors:** Per meet PRD §FR-02 calendar binding +
  participant join < 1.5s.
- **Edge cases:** Cross-pack media routing.
- **Error cases:** Camera permission.
- **Success outcome:** Call connects.
- **Failure recovery:** Audio-only fallback.

---

### Story Meet-02: Bob hosts an 8-person family reunion with background blur

- **Step-by-step actions:** Calendar → create event → "Add video" → invite 8.
- **Expected behaviors:** Per meet PRD §FR-09 breakout-not-needed; FR-19
  background blur; participant join flows.
- **Edge cases:** One attendee dials in.
- **Error cases:** SFU outage; auto-reconnect.
- **Success outcome:** Family reunion.
- **Failure recovery:** Rescheduled.

---

### Story Meet-03: Carol records a meeting for an absent friend

- Per meet PRD §FR-03 + recording with consent banner (KR PIPA Art. 15
  pattern even in personal pillar — modal banner shown to all participants).
- **Success outcome:** Recording link shareable.
- **Failure recovery:** Re-record; delete; retention controls.

---

### Story Meet-04: David joins on iPad, hands off to phone mid-call

- Per meet PRD §FR-05 multi-device; Apple Handoff or in-app
  device-switch token.

### Story Meet-05: Erin uses live captions in Portuguese

- Per FR-07; Whisper streaming; multilingual.

### Story Meet-06: Alice schedules a Meet room for monthly studio review

- Named room with stable URL; lobby on.

### Story Meet-07: Bob's family member dials in via phone

- Post-M03 dial-in (PSTN); pre-M03 web-only.

### Story Meet-08: Carol joins a public webinar (10k attendees) for photography

- Per FR-12 large-audience broadcast; HLS mesh.

### Story Meet-09: David tries E2E mode by accident (advisory)

- E2E disables transcription/recording; UX warns.

---

## 8. Surface 6 — Drive (personal)

### Story D-01: Alice uploads 500MB of vacation photos + creates a shared album

- **Persona:** Alice.
- **Step-by-step actions:** Drive → +Upload → Photos picker → 500MB.
- **Expected behaviors:** Per drive PRD §FR-01 multipart resumable; virus
  scan; preview rendered.
- **Edge cases:** HEIC; EXIF stripping options.
- **Error cases:** Quota; retries.
- **Success outcome:** Album shareable.
- **Failure recovery:** Resumed on reconnect.

---

### Story D-02: Bob backs up entire iPhone Photos library

- Drive desktop/mobile app's "Photos backup" toggle.

### Story D-03: Carol shares a folder with classmates via view-only link

- Per FR-04 share-link with password optional + expiry.

### Story D-04: David recovers a file from Trash

- Per FR-07; 30-day retention.

### Story D-05: Erin enables E2E personal vault

- Per FR-19 client-side E2E.

### Story D-06: Alice previews a PDF without download

- Per FR-09; gVisor sandbox PNG raster.

### Story D-07: Bob restores from a year-old version

- Per FR-23 versioning.

### Story D-08: Carol uses offline mode on a train

- Sync substrate; conflict resolution.

### Story D-09: David receives a malware warning on download

- Per FR-10 scan + quarantine.

### Story D-10: Erin transfers ownership of a folder to a colleague

- Per FR-15.

---

## 9. Surface 7 — Notes (personal)

### Story N-01: Alice writes a journal entry with rich text + photo + voice memo

- **Persona:** Alice.
- **Step-by-step actions:** Notes → "+ New note" → types title; uses
  inline image + audio attach.
- **Expected behaviors:** Per notes SDK plan §Capability Matrix; Loro
  CRDT for collab; voice memo stored via drive substrate.
- **Edge cases:** Offline write; conflict-free merge later.
- **Error cases:** Audio permission.
- **Success outcome:** Journal stored.
- **Failure recovery:** Offline retained.

---

### Story N-02: Bob creates an encrypted note for sensitive financial info

- Per MLS RFC 9420 personal-pillar E2E; key recovery via recovery phrase.

### Story N-03: Carol shares a note with a study group; real-time collab

- Loro CRDT.

### Story N-04: David uses daily-note auto-create with templates

- Per notes SDK plan daily-note + template materialisation.

### Story N-05: Erin imports notes from Obsidian (.md)

- Per notes SDK plan §Import (Obsidian/ENEX).

### Story N-06: Alice builds a graph view of her tag-graph

- Per notes SDK plan §Graph-view WebGL render.

### Story N-07: Bob web-clips an article via browser extension

- Per notes SDK plan §Web-clipper bridge.

### Story N-08: Carol turns checklist items into Tasks (via Workflow)

- Cross-µservice through Workflow events.

---

## 10. Surface 8 — Plugin App Store (personal)

### Story PAS-01: Alice browses for a "habit tracker" plugin

- **Persona:** Alice.
- **Step-by-step actions:** Open Plugin App Store → search "habit
  tracker" → filter by free → tap top result → review badges + screenshots.
- **Expected behaviors:** Per plugin-app-store PRD §FR-01-03; p95 ≤
  200ms search.
- **Edge cases:** Per-pack availability (some plugins blocked in
  pack-eu).
- **Error cases:** Search service slow; cached results shown.
- **Success outcome:** Found.
- **Failure recovery:** Search refinement.

---

### Story PAS-02: Alice installs the habit tracker with permission grant

- Per FR-04-05; capability grants modal; Cedar policy materialized; p95
  ≤ 5s install.
- **Edge cases:** Permission denied per capability; plugin runs reduced.
- **Error cases:** Cosign signature missing → blocked.
- **Success outcome:** Plugin live.
- **Failure recovery:** Uninstall one-tap.

---

### Story PAS-03: Bob purchases a premium plugin via credit card

- **Persona:** Bob.
- **Status:** *Post-payments-certification* (per ADR-0245 reserved
  µservice). Currently flagged as forward-looking.
- **Step-by-step actions:** Plugin → "$4.99/mo" → tap "Subscribe" →
  payment sheet (Apple Pay / card) → confirm.
- **Expected behaviors:** Reserved `payments` µservice; PCI-DSS scope;
  receipt + subscription managed via consolidated finops billing.
- **Edge cases:** Sub renews monthly; cancel anytime.
- **Error cases:** Card declined; alt payment.
- **Success outcome:** Premium features active.
- **Failure recovery:** Cancel one-tap; refund per policy.

---

### Story PAS-04: Carol reviews a plugin she's used for a month

- 5-star rating + text review; rate-limited; abuse-detected.

### Story PAS-05: David uninstalls a plugin that bothered him

- One-tap uninstall; data wipe; audit trail.

### Story PAS-06: Erin views per-plugin audit trail

- Per FR-12.

### Story PAS-07: Alice sets a per-plugin spend cap

- Per FR-15.

### Story PAS-08: Bob sees a plugin auto-suspended due to error rate

- Per FR-09 vetting + circuit breaker.

---

## 11. Surface 9 — Marketplace (personal)

*Marketplace µservice does not yet exist (per repo check 2026-05-20). All
stories below are **forward-looking** and marked post-certification
(payments + tax + IDV reserved µservices per ADR-0245).*

### Story MK-01: Carol buys a vintage camera from a C2C seller in Berlin (post-c2c-launch)

- Marketplace listing → buyer protection → checkout via reserved payments;
  C2C escrow; courier integration; review system.

### Story MK-02: Alice subscribes to a designer's exclusive content (Substack-equivalent; post-subscriptions-launch)

- Monthly subscription; in-app reader; cancel anytime; portable export.

### Story MK-03: Bob purchases a digital good (e-book)

- One-time purchase; DRM-light; portable EPUB export.

### Story MK-04: David lists a hobby cookbook for sale

- Seller onboarding via reserved IDV; payouts via reserved payments.

### Story MK-05: Erin reviews a marketplace seller

- Verified-purchase review; trust signals; appeal workflow.

### Story MK-06: Carol returns an item

- 14-day EU return per consumer rights; refund via reserved payments;
  audit-chain.

### Story MK-07: Alice tracks shipment status

- Carrier integration; cross-µservice via workflow events.

---

## 12. Cross-surface flows

### Story X-01: Alice receives a calendar invite via Mail → adds to Calendar → joins Meet

- Single click-through: mail item → calendar add → meet join — three
  µservices coordinate via Workflow events.

### Story X-02: Bob shares a Drive file via Messenger; recipient previews inline

- Drive share-link → messenger payload → recipient previews via drive
  preview BC.

### Story X-03: Carol mentions Erin in a Community post → notifications via Messenger + Mail digest

- Community `MentionEmitted` → notification routing → fan-out per Erin's
  prefs.

### Story X-04: David schedules a birthday Meet → invites via Calendar → reminders via Messenger

- Three-way orchestration.

### Story X-05: Alice exports vacation photos to a Notes journal entry

- Drive → Notes embed.

### Story X-06: Bob converts a mail into a Workflow task (consent-gated)

- Per mail PRD §FR-09 explicit handoff; audit-chain links.

### Story X-07: Carol federates a Mastodon post into her Community

- Per ActivityPub federation.

### Story X-08: David asks Siri to "schedule lunch with my son next week"

- Siri Intents → Calendar substrate.

### Story X-09: Erin uses Intelligence substrate to summarise a long meeting recording

- Meet recording → intelligence substrate summary → Notes pinned.

### Story X-10: Alice shares a sticker pack she designed to friends

- Messenger sticker pack → drive backing store → shareable deeplink.

### Story X-11: Bob receives a security alert across surfaces

- Cross-µservice security event → notification + email + in-app banner.

### Story X-12: Carol exports all her data (GDPR Art. 20) — cross-surface DSAR

- DSAR cascade per ADR-0242; bundles mail + drive + messenger + notes +
  calendar + community.

---

## 13. UX strive/avoid (cross-surface)

### 13.1 Strive

- 100ms response budget per gesture; 200ms p99 for navigation.
- Optimistic UI everywhere mutations are reversible.
- Undo affordance for 10s on every destructive action.
- Native gesture vocabularies per platform (Apple HIG; Material 3;
  Fluent 2; Carbon).
- Dark mode native + respects system setting.
- Dynamic Type / scalable type respected up to 200%.
- WCAG 2.2 AA day-one across every surface.
- ≥18 locales at GA; RTL + vertical text per locale.
- Offline-first read paths.
- Sync-on-reconnect with deterministic conflict resolution.
- Clear progress for any operation > 200ms.
- Emoji-friendly text rendering across Unicode 16.
- Hyperlink unfurl previews safe (no JS execution).
- Notifications batched + respect Focus / Do Not Disturb.
- Per-pillar isolation visible in UI (Personal vs Professional badges).
- Federated interop visible (Matrix / ActivityPub badges).

### 13.2 Avoid

- Dark patterns: unsubscribe hard, leave-community hidden, delete-account
  buried.
- Growth hacks: friction on departure; "Are you sure you don't want to
  rate us?" prompts.
- Engagement manipulation: infinite scroll without batch breaks.
- Attention-stealing notifications: zero badge / lockscreen text spam.
- Ad-style overlays: never.
- Pre-checked opt-ins.
- Surveillance-style tracking; no third-party SDKs in personal pillar.
- Cross-tenant data leakage in personal mode: structurally impossible
  per ADR-0242 + ADR-0244.
- "Continue with Google/Apple/Facebook" lock-in: passkeys + federated
  IdPs alternatives.
- Hidden subscription auto-renewals: always pre-warn ≥7 days.

---

## 14. Accessibility user stories

### Story A-01: Screen-reader user navigates Messenger

- **Persona:** A blind user using VoiceOver (iOS) or TalkBack (Android).
- **Step-by-step:** Swipe right through Chats list; double-tap to open;
  navigate messages; type with Braille screen input.
- **Expected behaviors:** Every element labelled; reading order matches
  visual order; focus moves predictably.
- **Edge cases:** Stickers / emoji read by alt-text; voice messages
  played; reactions announced.
- **Error cases:** Missing alt-text → fallback "unlabelled".
- **Success outcome:** Full functionality without sight.
- **Failure recovery:** Accessibility issues filed via in-app "Report
  accessibility" button.

### Story A-02: Color-blind user uses Calendar

- **Persona:** Deuteranopia.
- Calendar uses patterns + labels in addition to color; high contrast.

### Story A-03: Motor-impaired user uses voice input

- **Persona:** RSI; uses voice.
- All actions invocable via Voice Control (Apple) or Voice Access
  (Android).

### Story A-04: Low-vision user with 200% zoom

- All layouts reflow; no horizontal scroll.

### Story A-05: Cognitive-disability-friendly simplified mode

- "Simplified UI" toggle: fewer elements; larger targets; read-aloud
  buttons; plain-language strings.

---

## 15. Localization user stories

### Story L-01: Carol uses Hebrew RTL

- Full RTL mirroring; numerals localised; calendar week starts Sunday.

### Story L-02: Korean user with KakaoTalk-style sticker pack

- Local cultural conventions: bowing emoji set; pillar names in 한글.

### Story L-03: Japanese user with vertical text in Notes

- Optional vertical layout for poetry / traditional writing.

### Story L-04: Arabic RTL — Erin's source

- Bidirectional text rendering correct.

### Story L-05: Simplified vs Traditional Chinese variants per region

- Locale picker; ICU resource bundles.

---

## 16. References (2024-2026 UX sources)

- **Apple Human Interface Guidelines (2024 edition).** Comprehensive
  patterns; WWDC 2024 sessions 10010 (CloudKit), 10146 (SwiftUI
  navigation), 10073 (Accessibility for SwiftUI).
- **Material Design 3 (Google).** Material 3 spec; Material You dynamic
  colour; gesture patterns.
- **Microsoft Fluent 2 Design System (2024).** Cross-platform Fluent UI;
  Microsoft Build 2024 sessions.
- **IBM Carbon Design System v11 (2024-2025).** Enterprise patterns.
- **Reddit best practices (2024 product blog posts).** Voted-Q&A
  affordances; mod-tooling.
- **Discord engineering blog (2024).** Real-time presence + WebSocket
  scaling.
- **KakaoTalk product principles (KR; 2024 articles in Naver/Tistory
  engineering blogs).** Sticker rituals; Korean UX patterns.
- **LINE design guidelines (JP; 2024).** Sticker rituals; high-context
  notification.
- **WhatsApp business / consumer UX (Meta 2024 design summit posts).**
  Mass-scale messaging UX.
- **Signal blog 2024-2025.** Privacy-by-default messenger UX.
- **W3C WCAG 2.2 (October 2023, in widespread 2024-2026 adoption).**
- **W3C MediaCapabilities + Insertable Streams.** Used by Meet PRD §FR-15.
- **Matrix Foundation spec.matrix.org r0.6.1 + r0.1.4 LTS.**
- **ActivityPub W3C Recommendation.** Federation surface.
- **RFC 5545 / 5546 / 4791 (Calendar interop).**
- **RFC 5321 / 5322 / 8058 (Mail interop).**
- **AsyncAPI 3.0 (event contracts).**
- **OpenAPI 3.2 (REST contracts).**
- **Nielsen Norman Group articles 2024-2026 on dark patterns + consent UX.**
- **Apple WWDC 2024 sessions on UX privacy; iOS 18 + iPadOS 18 features.**
- **Google I/O 2024 sessions on Material You + Android multi-device UX.**
- **Google Cloud SRE Workbook ch. 2 (Beyer et al.) — SLO budget UX.**
- **GDPR Article 12 (DSAR response SLA), Article 17 (Erasure), Article
  20 (Data portability), Article 25 (Privacy by design + default).**
- **KR PIPA Article 15 / 21 / 22 / 36 / 39-4.**
- **EU AI Act 2024 (Article 17 high-risk classification; transparency
  duties).**
- **ADR-0242 (oyatie-is-a-tenant doctrine, 2026-05-20).**
- **ADR-0244 (tenant-as-universal-scoping-primitive, 2026-05-20).**
- **ADR-0245 (substrate-vs-product layering, 2026-05-20).**
- **ADR-0238 (dual-context dissolution; parallel-session).**
- **ADR-0220 / ADR-0255 (Intelligence substrate two-layer model).**
- **Mail PRD, Messenger PRD, Community PRD, Calendar PRD, Meet PRD,
  Drive PRD, Notes SDK plan, Plugin App Store PRD (this repo).**

---

## Appendix A: Story index by persona

| Persona | Story IDs |
|---|---|
| Alice | M-01, M-06, M-11, M-15 (referenced), M-16, M-20, Mail-01, Mail-06, Mail-13, Mail-18, C-03, C-06, C-10, Cal-01, Cal-06, Cal-11, Meet-01, Meet-06, D-01, D-06, N-01, N-06, PAS-01, PAS-02, PAS-07, MK-02, X-01, X-05, X-10 |
| Bob | M-02, M-07, M-12, Mail-02, Mail-07, Mail-12, Mail-14 (referenced), Mail-19 (referenced), C-02, Cal-02, Cal-09, Cal-12, Meet-02, Meet-07, D-02, D-07, N-02, N-07, PAS-03, PAS-08, MK-03, X-02, X-06, X-11 |
| Carol | M-03, M-08, M-13, M-17, Mail-03, Mail-08, Mail-11, Mail-16, C-01, C-07, C-11, Cal-03, Cal-07, Meet-03, Meet-08, D-03, D-08, N-03, N-08, PAS-04, MK-01, MK-06, X-03, X-07, X-12, A-01 (referenced), L-01 |
| David | M-04, M-09, M-14, M-18 (referenced), Mail-04, Mail-09, Mail-14, Mail-17, C-04, C-09, Cal-04, Cal-08, Meet-04, Meet-09, D-04, D-09, N-04, PAS-05, MK-04, X-04, X-08, L-03 |
| Erin | M-05, M-10, M-15, M-19, M-20 (referenced), Mail-05, Mail-10, Mail-15, Mail-20, C-05, C-08, C-12, Cal-05, Cal-10, Meet-05, D-05, D-10, N-05, PAS-06, MK-05, MK-07, X-09, L-04 |

---

## Appendix B: Story index by surface

| Surface | Story count | Story IDs |
|---|---|---|
| Messenger | 20 | M-01..M-20 |
| Mail | 20 | Mail-01..Mail-20 |
| Community | 12 | C-01..C-12 |
| Calendar | 12 | Cal-01..Cal-12 |
| Meet | 9 | Meet-01..Meet-09 |
| Drive | 10 | D-01..D-10 |
| Notes | 8 | N-01..N-08 |
| Plugin App Store | 8 | PAS-01..PAS-08 |
| Marketplace (post-cert) | 7 | MK-01..MK-07 |
| Cross-surface flows | 12 | X-01..X-12 |
| Accessibility | 5 | A-01..A-05 |
| Localization | 5 | L-01..L-05 |
| **Total** | **128** | |

---

## Appendix C: Open questions for product council

1. **Default Personal-pillar self-destruct timer pre-selection.** Should
   we default off (current) or default-warn for new users?
2. **AI-assist consent UX.** Per ADR-0255 Intelligence layering, default
   off + opt-in once per persona; should we explore opt-in per-use only?
3. **Federation onboarding visibility.** How prominent should Matrix /
   ActivityPub federation be on the new-user surface?
4. **B2C personal tenant ↔ enterprise tenant data sharing.** A user is
   both Alice-personal and Alice-at-Studio. How explicit is the persona
   switcher?
5. **Reserved-µservice forward language.** When showing Marketplace UI in
   roadmaps to consumers, what wording avoids creating expectations
   pre-certification?
6. **Localization quality bar by region.** What's the minimum
   acceptable string-coverage % at GA per locale?
7. **Children + COPPA / KR-Youth equivalent rules.** B2C personal tenant
   for users < 13: scope of feature ablation?

---

*End of compendium.*
