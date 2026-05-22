---
doc_class: User-Journey-Story
journey_id: j164-retired-hiroshi-tanaka-yearly-tax-and-pension
date: 2026-05-20
authority_tier: 2
status: draft
---

# j164 — Story: 09:14 JST Saturday in Kurashiki, the kotatsu and the tax form

## §0 — Saturday February 27, 2027, 09:14 JST — Kurashiki, Okayama

Late winter in the Sanyo region. Outside Hiroshi's house in Kurashiki the temperature is 4°C; a thin snow drift has settled on the persimmon tree (柿) and the stone lantern (灯籠) in the garden. The Mizushima factory chimneys in the distance are emitting their usual steam. The Seto Inland Sea is half a kilometer south but Hiroshi cannot see it from the house. Tama, his 9-year-old calico cat, is curled in the dent in the kotatsu blanket Sachiko used to sit in. The kerosene stove smells slightly of last winter.

Hiroshi (田中浩, 72歳, 倉敷市在住) has finished his breakfast: 焼き鯖 (grilled mackerel — half), 味噌汁 (miso soup with daikon + tofu), ご飯 (a smallish bowl of rice), たくあん (pickled daikon), and the second of three cups of 緑茶 (green tea). His feet are under the kotatsu. The clock on the kitchen wall — a tin clock from 1986 with Pokémon stickers his granddaughter Mio put on it in 2015 — reads 09:14:08 JST.

His Xiaomi Pad 6 Pro tablet is on the kotatsu beside Tama. The tablet is the one his daughter Misaki bought him at Yodobashi Camera in Akihabara in 2024-10. It has a 12.4" screen, set to brightness 70%, color temperature 5400K (warm), and the home screen renders Hiroshi's apps in **18pt bold Noto Sans CJK JP** on a **near-black background** (#0A0A0A with #F0F0F0 foreground — calibrated for his AMD-affected central vision). The TalkBack voice is set to female-voice-2 at speech rate 0.85x (slightly slower than default; Hiroshi finds this comfortable).

He taps the **workflow-studio** icon (the large blue icon his daughter labeled "ぜいきん" in hiragana so he can find it without reading). TalkBack announces:

> "ワークフロー・スタジオを開いています。お待ちください。"
> ("Opening Workflow Studio. Please wait.")

The tablet's haptic motor pulses gently — one short pulse to confirm the tap, then a long pulse to confirm the app opened. The active-tenant pill at the top of the screen reads `personal-hiroshi-tanaka-jp` in large hiragana + romaji and is rendered with a black background and a thin gold border (the high-contrast theme).

`EVT-J164-WORKFLOW-OPEN-001` sealed at 09:14:42 JST.

## §1 — 09:14–09:42 JST: opening the annual tax-prep workflow

The workflow-studio canvas opens to Hiroshi's pinned workflow: **"令和8年度・確定申告" (FY2026 kakutei-shinkoku)**. The workflow card shows:

```
┌─────────────────────────────────────────────────┐
│  令和8年度 (FY2026)  確定申告                    │
│  ────                                            │
│  期限: 令和9年3月15日 (March 15, 2027)           │
│  残り日数: 16日                                  │
│                                                  │
│  進捗: ▓░░░░░░░░  10% (started)                 │
│                                                  │
│  前回 (令和7年度) との比較を見る                  │
│                                                  │
│  [ ステップ1: 領収書を集める     未着手 ]         │
│  [ ステップ2: 年金の照合         未着手 ]         │
│  [ ステップ3: 支払いの照合       未着手 ]         │
│  [ ステップ4: 申告書の下書き     未着手 ]         │
│  [ ステップ5: 確認               未着手 ]         │
│  [ ステップ6: e-Tax で提出       未着手 ]         │
│                                                  │
│  助言: 大きな声で「進む」と言うと次のステップへ │
│        移動します。                              │
└─────────────────────────────────────────────────┘
```

TalkBack reads the card. The voice is unhurried. Hiroshi listens and nods once. He says aloud, in normal voice volume:

> "進む" ("Susumu" — proceed)

The tablet's voice-navigation substrate recognizes the command. Voice-recognition confidence 0.94. The substrate engages step 1.

## §2 — 09:42–10:48 JST: collecting the receipts (drive)

Step 1 is **領収書を集める** (collect the receipts). Hiroshi has been saving his 2026 receipts in a manila envelope on the genkan shelf for the past 12 months. Sachiko had a system: she would empty the wallet every Sunday morning and file receipts immediately. Hiroshi has tried to maintain her system but it has loosened over the years. The envelope contains a mix of carefully-folded receipts and crumpled ones.

He carries the envelope to the kotatsu. He sits down. Tama purrs.

The workflow surfaces the **receipt collection screen**:

```
┌─────────────────────────────────────────────────┐
│  領収書を集める (令和8年度)                       │
│  ────                                            │
│  カメラで撮ってください。一枚ずつで構いません。 │
│  写真は自動で読み取り (OCR) されます。           │
│                                                  │
│  [ 大きなカメラボタン ]   ← 押す                 │
│                                                  │
│  または:                                         │
│  [ 銀行口座から自動取り込み ]                    │
│  [ クレジットカードから自動取り込み ]            │
│                                                  │
│  これまでに集めた領収書: 0件                     │
└─────────────────────────────────────────────────┘
```

Hiroshi prefers the camera. He picks up the first receipt — an ophthalmology receipt from Kurashiki Central Hospital (倉敷中央病院) dated 2026-01-18 for ¥4,200. He places it on the kotatsu under the tablet's camera light. The camera fires. The OCR runs. TalkBack reads back:

> "倉敷中央病院。診療日: 令和8年1月18日。金額: 4,200円。眼科。受領しました。次の領収書をどうぞ。"

The intelligence µservice OCR has correctly parsed the kanji + the date + the amount. Hiroshi nods. He puts the receipt aside in a "done" pile.

He works through 17 receipts over the next 64 minutes. He takes a tea break at 10:14 (his hands are not as steady as they used to be; the camera focus needs a steady hand) and again at 10:38. The full list:

| Receipt # | Date | Payee | Amount | Category |
|---|---|---|---|---|
| 1 | 2026-01-18 | 倉敷中央病院 眼科 | ¥4,200 | medical |
| 2 | 2026-02-22 | 倉敷中央病院 眼科 | ¥4,200 | medical |
| 3 | 2026-04-08 | はやし整形外科 (Hayashi Orthopedics) | ¥3,800 | medical |
| 4 | 2026-04-18 | 倉敷中央病院 眼科 | ¥4,800 | medical |
| 5 | 2026-05-14 | やまもと歯科 (Yamamoto Dental) | ¥6,200 | medical |
| 6 | 2026-06-12 | 倉敷中央病院 眼科 | ¥4,200 | medical |
| 7 | 2026-07-04 | 倉敷中央病院 循環器科 | ¥18,400 | medical |
| 8 | 2026-08-22 | 倉敷中央病院 眼科 | ¥4,800 | medical |
| 9 | 2026-09-18 | やまもと歯科 | ¥38,000 | medical (implant) |
| 10 | 2026-10-14 | 倉敷中央病院 眼科 | ¥4,200 | medical |
| 11 | 2026-11-08 | 倉敷中央病院 眼科 | ¥4,200 | medical |
| 12 | 2026-12-12 | 倉敷中央病院 眼科 + 循環器科 | ¥25,400 | medical |
| 13 | 2026-05-22 | 倉敷市 固定資産税 1期 | ¥34,800 | property tax |
| 14 | 2026-07-22 | 倉敷市 固定資産税 2期 | ¥34,800 | property tax |
| 15 | 2026-09-22 | 倉敷市 固定資産税 3期 | ¥34,800 | property tax |
| 16 | 2026-12-22 | 倉敷市 固定資産税 4期 | ¥34,800 | property tax |
| 17 | 2026-11-04 | 三菱重工 OB会 寄稿料 (alumni honorarium) | ¥20,000 | other income |

Medical total: ¥126,400. Property tax total: ¥139,200. Honorarium total: ¥20,000.

`EVT-J164-RECEIPTS-COLLECTED-002` sealed at 10:48:14 JST. Receipts archived to `drive` room `personal-hiroshi-tanaka-jp/tax/fy2026/receipts/`. WORM lock engaged with 7-year retention timer (Japan tax authority requires 7-year retention for individual tax records under 所得税法 第148条 + 法人税法 retention rules adapted for individuals).

## §3 — 10:48–11:24 JST: pension reconciliation (payments)

Hiroshi takes a short break — he walks to the entryway, looks at the snow on the persimmon tree, and pets Tama who has followed him. Back at the kotatsu at 10:54.

The workflow advances to step 2: **年金の照合** (pension reconciliation).

The payments µservice queries the JPS (日本年金機構, Japan Pension Service) direct-deposit feed. Hiroshi authenticates with My-Number Card NFC. The first tap fails (his hands are unsteady; the tablet's NFC reader needs the card held against the back for ~2 seconds). The screen displays a calm, large-text instruction:

```
マイナンバーカードを背面にかざしてください。
動かさないでください。
3秒間そのまま保持してください。
```

The tablet uses a 30-second timeout — generous, no anxiety-induction. Hiroshi tries again. The second tap also fails (he angled the card wrong). On the third tap at 10:56:42 JST the card reads successfully. TalkBack announces:

> "マイナンバーカードを認識しました。年金記録を取得しています。お待ちください。"

`EVT-J164-MY-NUMBER-NFC-007` sealed at 10:56:42 JST. My-Number access purpose declared: `pension_reconciliation`.

The payments µservice returns the pension record:

```
日本年金機構 年金記録 令和8年度
────
受給者: 田中 浩
基礎年金番号: ****-****-1234 (last 4 digits shown only)
受給開始: 平成31年4月 (April 2019, age 65)

月別振込:
2026-01-15  ¥182,000  (1月分)
2026-02-15  ¥182,000  (2月分)
2026-03-13  ¥182,000  (3月分)
2026-04-15  ¥182,000  (4月分)
2026-05-15  ¥182,000  (5月分)
2026-06-15  ¥182,000  (6月分)
2026-07-15  ¥182,000  (7月分)
2026-08-14  ¥182,000  (8月分)
2026-09-15  ¥182,000  (9月分)
2026-10-15  ¥182,000  (10月分)
2026-11-13  ¥182,000  (11月分)
2026-12-15  ¥182,000  (12月分)
────
年間合計: ¥2,184,000
源泉徴収: ¥56,750
```

The payments µservice cross-checks this against the Chugoku Bank (中国銀行) deposit feed. 12/12 matches at the exact dates + amounts. No discrepancies.

TalkBack reads back the annual total. Hiroshi confirms with the voice command "確認" (kakunin — confirm).

`EVT-J164-PENSION-RECONCILED-003` sealed at 11:24:08 JST.

## §4 — 11:24–11:54 JST: tax payment reconciliation (payments)

The workflow advances to step 3: **支払いの照合** (tax payment reconciliation). The payments µservice queries Hiroshi's quarterly estimated tax payment history:

```
予定納税 (estimated quarterly tax) 令和8年度
────
2026-07-31  ¥18,400  (第1期分)
2026-10-31  ¥18,400  (第2期分)
2027-01-31  ¥18,400  (第3期分)
(注: 第4期分は確定申告で精算)
────
合計: ¥55,200
```

Plus property tax (already in receipts), national health insurance (国民健康保険), and介護保険 (long-term care insurance):

- 国民健康保険: ¥84,000 (monthly ¥7,000 × 12)
- 介護保険: ¥48,000 (monthly ¥4,000 × 12)

The payments µservice reconciles against Hiroshi's bank statement; all 12 monthly direct debits clear. `EVT-J164-TAX-PAYMENTS-RECONCILED-004` sealed at 11:54:18 JST.

## §5 — 11:54–12:24 JST: lunch break

Hiroshi pauses the workflow with the voice command "休憩" (kyukei — break). The workflow saves state to `personal-hiroshi-tanaka-jp` (durable; survives tablet reboot). TalkBack acknowledges:

> "ワークフローを一時停止しました。準備ができたら『続ける』とお声がけください。"

He makes lunch: leftover mackerel reheated, a small portion of rice, more miso soup, a salty plum (梅干し). Tama gets a small piece of the mackerel skin. The kerosene stove ticks softly. The snow in the garden has stopped; weak February sunlight is now angling through the shoji.

At 12:18 his son Daiki calls from Osaka. They talk for 6 minutes — Hiroshi mentions he's doing the tax form, Daiki says "気をつけて、急がないで" (be careful, don't rush). Hiroshi appreciates this.

At 12:24:18 JST Hiroshi says "続ける" (tsuzukeru — continue). The workflow resumes.

## §6 — 12:24–13:18 JST: year-over-year comparison + form drafting (workflow-studio)

The workflow advances to step 4: **申告書の下書き** (draft the form). Before drafting, the workflow presents the **year-over-year comparison panel** — a TalkBack-friendly side-by-side panel that Hiroshi's daughter Misaki built into his workflow 2 years ago because she wanted him to see at a glance whether anything had changed:

```
前年度比較 (FY2025 vs FY2026)
────
収入                FY2025          FY2026          差額
  年金             ¥2,184,000      ¥2,184,000      ±¥0
  原稿料                  -          ¥20,000      +¥20,000
  銀行利息            ¥3,200          ¥3,200        ±¥0
  合計             ¥2,187,200      ¥2,207,200    +¥20,000

控除                FY2025          FY2026          差額
  医療費             ¥98,400         ¥126,400      +¥28,000
  社会保険料        ¥132,000        ¥132,000        ±¥0
  基礎控除           ¥480,000        ¥480,000        ±¥0
  配偶者控除         ¥0 (死別)        ¥0 (死別)       -

源泉徴収            FY2025          FY2026          差額
  年金分             ¥56,750         ¥56,750        ±¥0
  予定納税           ¥73,600         ¥55,200      -¥18,400

推定税額           FY2025          FY2026          差額
  確定税額           ¥51,400         ¥37,400      -¥14,000
  推定還付額        +¥5,350         +¥17,800      +¥12,450
```

TalkBack reads each row. The panel has voice-command "詳しく" (kuwashiku — more detail) per row. Hiroshi asks for more detail on the medical row — the workflow zooms in and shows:

```
医療費控除 (medical expense deduction)
────
医療費合計:        ¥126,400
控除最低限度額:    ¥100,000
控除対象額:         ¥26,400 (= ¥126,400 - ¥100,000)
前年度控除対象:     ¥0 (医療費が¥100,000未満だった)
────
新しい控除: ¥26,400
これは推定税額を約¥2,640 減らします。
```

Hiroshi nods. He says "下書きを進める" (proceed to draft).

The workflow drafts the kakutei-shinkoku form. The form is rendered in large-text TalkBack-friendly format:

```
令和8年度 確定申告書 (B様式) — 下書き
────
氏名: 田中 浩
住所: 岡山県倉敷市美和2丁目14-7
生年月日: 昭和29年11月8日
マイナンバー: 取得済み (表示はしません)

収入:
  公的年金 (国民年金 + 厚生年金)     ¥2,184,000
  雑所得 (原稿料)                       ¥20,000
  利子所得                               ¥3,200
  ─                                  ─────────
  合計                              ¥2,207,200

所得控除:
  社会保険料控除                       ¥132,000
  医療費控除                            ¥26,400
  基礎控除                             ¥480,000
  ─                                  ─────────
  合計                                ¥638,400

課税所得:                            ¥1,568,800
  (= 収入 ¥2,207,200 - 公的年金等控除 ¥0 - 所得控除 ¥638,400)

税額計算:
  所得税                                ¥37,400
  復興特別所得税 (2.1%)                     ¥785
  ─                                  ─────────
  合計税額                              ¥38,185

源泉徴収済み:
  年金分                                ¥56,750
  予定納税                              ¥55,200
  ─                                  ─────────
  合計                                ¥111,950

差引                                  -¥73,765 (還付)
還付予定額: ¥73,765
```

The form has gone better than the year-over-year preview suggested — the predicted refund was ¥17,800 but the actual draft shows ¥73,765 because the workflow correctly applied the public pension deduction (公的年金等控除) which the YoY preview had simplified out for at-a-glance display. TalkBack reads back the refund line.

Hiroshi says "ありがとう、保存して" (thank you, save it).

`EVT-J164-FORM-DRAFTED-006` sealed at 13:18:42 JST.

## §7 — 13:18–13:42 JST: review

The workflow advances to step 5: **確認** (review). The form is presented section by section. TalkBack reads each section. Hiroshi listens and confirms each. At each confirmation the haptic motor pulses gently — a tactile signal that his confirmation registered.

At 13:38 he flags one concern: he is not sure whether the alumni honorarium counts as "雑所得 (zatsushotoku — miscellaneous income)" or "事業所得 (jigyoshotoku — business income)". The workflow handles this via the in-context help:

```
お問い合わせ:
  原稿料 ¥20,000 は雑所得として記載されています。
  これは正しい区分です。

  理由:
   • 年に1回の単発の寄稿である
   • 事業として継続していない
   • ¥30万円以下である
   • 三菱重工OB会という非営利団体からの謝礼である

  もし継続的な事業として執筆活動をしている場合は
  事業所得になりますが、お客様の場合は雑所得が正しいです。
```

Hiroshi nods. He says "良し、進む" (yoshi, susumu — good, proceed).

`EVT-J164-FORM-REVIEW-006a` sealed at 13:42:08 JST.

## §8 — 13:42–14:08 JST: My-Number tap + final form lock

The workflow advances to step 6: **e-Tax で提出** (submit via e-Tax). This step requires another My-Number Card NFC tap (the second of the day; My-Number per-access count is now 2). Hiroshi takes the card out of his wallet, places it against the back of the tablet, and holds steady. This time it reads on the first try at 13:48:18 JST.

My-Number purpose declaration: `etax_submission`. The third per-purpose access is logged.

The form is locked for submission. A final review modal appears:

```
提出前の最終確認
────
これから国税庁の e-Tax システムに送信します。
送信後の変更には修正申告が必要です。

  送信内容:
   • 申告書本体 (3.2 MB)
   • 領収書添付 (17件)
   • 年金記録 (国税庁が照合できるよう参照)

  電子署名: マイナンバーカード + パスキー
   ✓ マイナンバーカード認証済み
   ✓ パスキー認証 (顔認証またはPIN) を求めます

  税額確定: ¥38,185 (還付予定 ¥73,765)
```

Hiroshi places his face in front of the tablet camera. Face-ID authentication (he opted into face-ID at his daughter's recommendation in 2024) succeeds in 1.4 seconds. The passkey assertion completes.

## §9 — 14:08–14:36 JST: e-Tax submission

The compliance µservice opens the e-Tax linkage channel. The submission travels over HTTPS-over-QUIC to the 国税庁 (NTA — National Tax Agency) endpoint via Hiroshi's `jp-tokyo-etax-linkage-readonly` cell. The transmission takes 24 seconds (3.2 MB + 17 receipts + signed metadata).

At 14:14:42 JST the NTA acknowledgment receipt arrives:

```
国税庁 e-Tax 受付通知
────
受付番号: 20270227-1414-008-T-7842965
受付日時: 令和9年2月27日 14時14分42秒
申告者: 田中 浩 (****-****-1234)
申告区分: 令和8年度 確定申告書 (B様式)
還付予定額: ¥73,765
還付予定日: 令和9年4月上旬

この受付通知は確定した受付の証明として保管してください。
```

The acknowledgment is archived to `drive` room `personal-hiroshi-tanaka-jp/tax/fy2026/submission/` with WORM 7-year retention. The receipt PDF is signed by the NTA + counter-signed by Hiroshi's tenant audit chain.

`EVT-J164-ETAX-SUBMITTED-009` sealed at 14:14:48 JST.

The workflow advances to terminal state: **完了** (complete). The progress bar goes to 100%. TalkBack announces:

> "確定申告が完了しました。受付番号は 20270227-1414-008-T-7842965 です。還付予定額は7万3千7百65円です。お疲れさまでした。"

The haptic motor pulses three times — long, long, long — the celebration pattern Hiroshi's grandson Hayate programmed for him.

## §10 — 14:36 JST: the diary entry (notes)

Hiroshi closes the workflow. He opens the **notes** app and selects his late-life record-keeping notebook. He uses voice dictation. He says:

> "令和9年2月27日。確定申告を提出した。還付は7万3千7百65円。タマは元気。みさきから明後日電話するそうだ。サチコへ — 今年もちゃんと終わらせました。あなたが見ていてくれているといいな。"
>
> ("February 27, 2027. Filed the tax return. Refund ¥73,765. Tama is fine. Misaki said she'll call the day after tomorrow. Sachiko — I finished it properly again this year. I hope you're watching.")

The notes µservice transcribes the dictation. The voice recognition accuracy is 96% (the difficult name "サチコ" is captured correctly — the system has Sachiko in Hiroshi's contact graph because he has dictated her name many times before). The entry is archived to the late-life notebook with the succession-ready tag. Per ADR-0311 § personal-tenant-succession, this notebook is pre-tagged for transfer to Misaki + Daiki under Hiroshi's existing succession plan; Hiroshi himself has not signed the succession plan yet (he keeps meaning to), but the notebook is ready when he does.

`EVT-J164-NOTES-DIARY-008` sealed at 14:36:18 JST.

## §11 — 14:36 JST: end-of-day + the long view

Hiroshi closes the tablet. He stands up — slowly; his knees object — and walks to the kitchen. He puts the kettle on for a fresh cup of tea. Tama follows him. The kerosene stove ticks. The shoji is glowing with the weak late-winter sunset.

The 確定申告 is done for the year. The refund will arrive in early April — usually around April 8 in his experience. The receipt for the NTA submission is archived to drive with 7-year retention. The diary entry is in his late-life notebook. The four My-Number accesses today are logged with declared purposes (workflow_open didn't actually access My-Number; only pension_reconcile, etax_authenticate, etax_submit — so three accesses; the count is within the daily ceiling of 4).

He thinks about Sachiko. She was the better record-keeper. She would have finished the form by 11 AM and made onigiri for lunch. He smiles. He pours the tea.

## §12 — Beats not on the wire (the human texture)

- At 09:14 when the workflow opened, Hiroshi's first instinct was to read the entire welcome message visually, then he caught himself and let TalkBack read it. He has been training himself for 18 months to let the assistive tech do its job instead of straining his AMD-affected vision.
- At 10:14 his tea break, Tama jumped onto his lap and bumped the camera angle slightly, causing one OCR failure on receipt #5 (the dental). The system politely asked him to retry; he did; it worked. The system did not blame him or imply any failure.
- At 11:54 the third My-Number Card tap that finally worked, Hiroshi's hand was shaking a little because his blood-pressure meds make his hands tremor in cold weather. He took the kerosene stove's position into account on the next tap.
- At 12:18 his son Daiki's call, Daiki had wanted to suggest his father use a tax accountant this year (they had had this conversation in 2025 and 2026 too); Daiki decided not to push it after hearing Hiroshi's voice — Hiroshi sounded clear and competent.
- At 13:42 the alumni-honorarium-vs-business-income decision, Hiroshi remembered the three editors at the OB会 newsletter by name (Yamada-san, Watanabe-san, the new young one whose name he forgets). They all liked his Mizushima reminiscences article.
- At 14:36 the diary entry to Sachiko, this is the 4th anniversary year since her death. He has done this exact diary pattern every February 27 since 2024. The system knows; the notes µservice tagged the entry as continuing the "annual-letter-to-sachiko" series.

## §13 — Stop condition for this story

This story documents the lived texture of the 5h22m journey from 09:14 JST tablet open through 14:36 JST diary entry to Sachiko. The acceptance criteria in `README.md`, the API shapes in `handshake.md`, the test cases in `integration-test-plan.md`, and the schemas together encode machine semantics. The story exists so the next reader understands WHY the system was patient with the failed NFC taps without ever surfacing failure language, WHY the year-over-year comparison was a first-class capability rather than a hidden report, WHY the My-Number per-purpose scoping mattered for PIPA compliance even when Hiroshi never explicitly thought about it, and WHY the late-life diary entry is treated as first-class data that will eventually flow through a succession plan rather than as an ephemeral note.
