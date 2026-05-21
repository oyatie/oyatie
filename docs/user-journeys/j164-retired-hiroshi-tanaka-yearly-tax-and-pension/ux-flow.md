---
doc_class: User-Journey-UX-Flow
journey_id: j164-retired-hiroshi-tanaka-yearly-tax-and-pension
date: 2026-05-20
authority_tier: 2
status: draft
---

# j164 — UX flow: TalkBack-first, voice navigation, high-contrast, large-text

Primary surface: Hiroshi's Xiaomi Pad 6 Pro (12.4" screen; high-contrast theme; 18pt minimum body text; TalkBack female-voice-2 at 0.85x speed; voice navigation always on; haptic feedback on every action).

Design principles for this journey:

- **TalkBack reads everything** — no purely-visual affordances; every UI element has a TalkBack annotation that is the equivalent (not a degraded fallback)
- **Voice command is first-class** — "進む" (proceed), "戻る" (back), "確認" (confirm), "詳しく" (detail), "休憩" (break), "続ける" (continue), "ありがとう" (thanks — exits with grace)
- **Haptic confirms every action** — short pulse on tap acknowledge; long pulse on action complete; triple long pulse on success milestone
- **Never use red for errors** (Hiroshi's AMD reduces red sensitivity); use orange + bold + the TalkBack voice slowing slightly
- **30-second timeouts everywhere** — never a 5-second timeout that creates anxiety; the system is patient
- **No "failed" or "error" language** — use "もう一度どうぞ" (please try once more) or "お待ちください" (please wait)

## Screen 1 — Tablet home screen + workflow icon (09:14 JST)

```
┌──────────────────────────────────────────────────────────────────────┐
│  active tenant: personal-hiroshi-tanaka-jp                            │
│  ─────                                                                │
│                                                                       │
│  おはようございます、ひろし さん。                                     │
│  Good morning, Hiroshi.                                               │
│                                                                       │
│   ┌────────────────────────────────────────────────────────────────┐ │
│   │                                                                │ │
│   │   ┌─────────────────┐    ┌─────────────────┐                  │ │
│   │   │     ぜいきん     │    │      ろぐ        │                  │ │
│   │   │  (tax-prep)     │    │   (diary)       │                  │ │
│   │   │   workflow      │    │   notes         │                  │ │
│   │   └─────────────────┘    └─────────────────┘                  │ │
│   │                                                                │ │
│   │   ┌─────────────────┐    ┌─────────────────┐                  │ │
│   │   │   れんらく       │    │    しんりょう     │                  │ │
│   │   │   (call family) │    │   (medical)     │                  │ │
│   │   └─────────────────┘    └─────────────────┘                  │ │
│   │                                                                │ │
│   └────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│   ⓘ いつでも『たすけて』とお声がけください。サポートを受けられます。  │
│     ("Say 'help' anytime to get support.")                            │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

TalkBack annotations:

- "Good morning Hiroshi-san. Four buttons on screen. Tax-prep workflow. Diary notes. Call family. Medical. Say 'help' anytime."
- App icons labeled in hiragana for read-aloud clarity (`ぜいきん` rather than `税金`) AND have full kanji visible for sighted use — both layers exist.
- High-contrast colors: black background, gold border on focused icon, white-on-black text.
- Body text ≥ 18pt; icon labels ≥ 24pt bold.

## Screen 2 — Workflow card (09:14 JST after tap)

```
┌──────────────────────────────────────────────────────────────────────┐
│  令和8年度 (FY2026)  確定申告                                          │
│  ─────                                                                │
│                                                                       │
│  期限: 令和9年3月15日 (March 15, 2027)                                 │
│  残り日数: 16日                                                        │
│                                                                       │
│  進捗: ▓░░░░░░░░  10% (開始)                                          │
│                                                                       │
│  前回 (令和7年度) との比較を見る  [大ボタン]                            │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │  ステップ1: 領収書を集める                       未着手            │ │
│  │  ステップ2: 年金の照合                          未着手            │ │
│  │  ステップ3: 支払いの照合                        未着手            │ │
│  │  ステップ4: 申告書の下書き                      未着手            │ │
│  │  ステップ5: 確認                                未着手            │ │
│  │  ステップ6: e-Tax で提出                        未着手            │ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│  ⓘ 助言: 大きな声で「進む」と言うと次のステップへ移動します。           │
│  ("Hint: Say 'proceed' loudly to move to the next step.")             │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

TalkBack reads the card section-by-section. Voice commands listed in the hint.

## Screen 3 — Receipt collection (10:14 JST mid-receipt)

```
┌──────────────────────────────────────────────────────────────────────┐
│  領収書を集める (令和8年度)                                            │
│  ─────                                                                │
│                                                                       │
│  カメラで撮ってください。一枚ずつで構いません。                        │
│  写真は自動で読み取り (OCR) されます。                                 │
│                                                                       │
│   ┌──────────────────────────────────────────────────────────────┐   │
│   │                                                              │   │
│   │      ┌──────────────────────────┐                            │   │
│   │      │                          │                            │   │
│   │      │  [大きなカメラボタン]    │                            │   │
│   │      │                          │                            │   │
│   │      └──────────────────────────┘                            │   │
│   │                                                              │   │
│   │   または:                                                    │   │
│   │   [銀行口座から自動取り込み]                                  │   │
│   │   [クレジットカードから自動取り込み]                          │   │
│   │                                                              │   │
│   └──────────────────────────────────────────────────────────────┘   │
│                                                                       │
│  ─ これまで集めた領収書: 5件 ─                                         │
│   ✓ 倉敷中央病院 眼科   令和8年1月18日   4,200円                       │
│   ✓ 倉敷中央病院 眼科   令和8年2月22日   4,200円                       │
│   ✓ はやし整形外科     令和8年4月8日   3,800円                         │
│   ✓ 倉敷中央病院 眼科   令和8年4月18日   4,800円                       │
│   ✓ やまもと歯科       令和8年5月14日   6,200円                        │
│                                                                       │
│   小計: 23,200円                                                      │
│                                                                       │
│  ⓘ 終わったら『おわった』と言ってください。                            │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

UX notes:

- Camera button is the largest UI element on the screen — 240×240px.
- Each collected receipt has a checkmark + payee + date + amount, large-text, read by TalkBack on each new receipt.
- Running subtotal updates after each receipt.
- Camera failure (Tama bump) surfaces "もう一度どうぞ" not "error" — no blame language.
- Voice command "おわった" (owatta — done) advances to next step.

## Screen 4 — Year-over-year comparison (12:48 JST)

```
┌──────────────────────────────────────────────────────────────────────┐
│  前年度比較 (FY2025 vs FY2026)                                         │
│  ─────                                                                │
│                                                                       │
│  ┌─ 収入 ─────────────────────────────────────────────────────────┐  │
│  │  項目         FY2025         FY2026        差額                 │  │
│  │  年金        ¥2,184,000    ¥2,184,000     ±¥0                  │  │
│  │  原稿料           -          ¥20,000     +¥20,000               │  │
│  │  銀行利息       ¥3,200       ¥3,200       ±¥0                  │  │
│  │  合計        ¥2,187,200   ¥2,207,200   +¥20,000                │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─ 控除 ─────────────────────────────────────────────────────────┐  │
│  │  項目         FY2025         FY2026        差額                 │  │
│  │  医療費        ¥98,400      ¥126,400    +¥28,000               │  │
│  │   控除対象       ¥0          ¥26,400    +¥26,400               │  │
│  │  社会保険料   ¥132,000      ¥132,000      ±¥0                  │  │
│  │  基礎控除     ¥480,000      ¥480,000      ±¥0                  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─ 源泉徴収 ─────────────────────────────────────────────────────┐  │
│  │  項目         FY2025         FY2026        差額                 │  │
│  │  年金分        ¥56,750       ¥56,750      ±¥0                  │  │
│  │  予定納税      ¥73,600       ¥55,200     -¥18,400               │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─ 推定税額 ─────────────────────────────────────────────────────┐  │
│  │  項目         FY2025         FY2026        差額                 │  │
│  │  確定税額      ¥51,400       ¥37,400     -¥14,000               │  │
│  │  推定還付額    +¥5,350      +¥17,800    +¥12,450               │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ⓘ 「詳しく」と言うと、各行を詳しく説明します。                        │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

UX notes:

- Side-by-side columns with explicit delta column.
- TalkBack reads each row including the delta direction (`+¥20,000` is read as "プラス2万円", `±¥0` as "差額なし").
- Voice command "詳しく" (kuwashiku — more detail) zooms into the highlighted row.
- The "原稿料" (honorarium) appearing for the first time is highlighted with a thin gold underline as a meaningful change.

## Screen 5 — Form draft (13:18 JST)

```
┌──────────────────────────────────────────────────────────────────────┐
│  令和8年度 確定申告書 (B様式) — 下書き                                  │
│  ─────                                                                │
│                                                                       │
│  氏名:     田中 浩                                                    │
│  住所:     岡山県倉敷市美和2丁目14-7                                  │
│  生年月日: 昭和29年11月8日                                            │
│  マイナンバー: 取得済み (表示はしません)                              │
│                                                                       │
│  ┌─ 収入 ─────────────────────────────────────────────────────────┐  │
│  │  公的年金 (国民年金 + 厚生年金)            ¥2,184,000           │  │
│  │  雑所得 (原稿料)                              ¥20,000           │  │
│  │  利子所得                                       ¥3,200           │  │
│  │  ──                                          ─────────           │  │
│  │  合計                                       ¥2,207,200           │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─ 所得控除 ─────────────────────────────────────────────────────┐  │
│  │  社会保険料控除                                ¥132,000          │  │
│  │  医療費控除                                     ¥26,400          │  │
│  │  基礎控除                                      ¥480,000          │  │
│  │  ──                                          ─────────           │  │
│  │  合計                                         ¥638,400           │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─ 税額計算 ─────────────────────────────────────────────────────┐  │
│  │  課税所得 (= 収入 - 控除)                  ¥1,568,800           │  │
│  │  所得税                                       ¥37,400            │  │
│  │  復興特別所得税 (2.1%)                            ¥785            │  │
│  │  ──                                          ─────────           │  │
│  │  合計税額                                      ¥38,185            │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─ 源泉徴収済み + 予定納税 ─────────────────────────────────────┐  │
│  │  年金分                                        ¥56,750           │  │
│  │  予定納税                                      ¥55,200           │  │
│  │  ──                                          ─────────           │  │
│  │  合計                                         ¥111,950           │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌─ 還付予定額 ────────────────────────────────────────────────────┐  │
│  │                                                                 │  │
│  │     ¥73,765  (還付)                                              │  │
│  │                                                                 │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ⓘ 各項目について質問があれば「詳しく」と言ってください。              │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

## Screen 6 — My-Number Card NFC patient retry (10:54 JST)

```
┌──────────────────────────────────────────────────────────────────────┐
│  マイナンバーカードを読み取ります                                      │
│  ─────                                                                │
│                                                                       │
│  目的: 年金記録の照合                                                  │
│  この目的以外には使用しません。                                        │
│                                                                       │
│   ┌──────────────────────────────────────────────────────────────┐   │
│   │                                                              │   │
│   │     カードを画面の背面に                                     │   │
│   │     しっかり当ててください                                   │   │
│   │                                                              │   │
│   │     [📱 タブレットの背面]                                    │   │
│   │                                                              │   │
│   │     動かさないでください                                     │   │
│   │     3秒間そのまま保持してください                            │   │
│   │                                                              │   │
│   └──────────────────────────────────────────────────────────────┘   │
│                                                                       │
│  ⏱ 残り時間: 28秒                                                     │
│                                                                       │
│  ⓘ もし上手くいかない時は焦らずもう一度ためしてください。              │
│    ("If it doesn't work, please don't rush — try again calmly.")     │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

UX notes:

- Patient language throughout: no "失敗" (failure); use "もう一度どうぞ" (please try once more).
- 30-second timeout — generous; never anxiety-inducing.
- Purpose is declared up front + a reassurance that the My-Number won't be used for other purposes.
- Haptic feedback: tap acknowledge on first contact; long-confirm pulse on successful read; gentle short pulse on retry-needed (NOT a sharp buzz).

## Screen 7 — e-Tax submission confirmation (14:14 JST)

```
┌──────────────────────────────────────────────────────────────────────┐
│  国税庁 e-Tax 受付通知                                                 │
│  ─────                                                                │
│                                                                       │
│   ✓ 受付完了しました                                                  │
│                                                                       │
│  受付番号: 20270227-1414-008-T-7842965                                 │
│  受付日時: 令和9年2月27日 14時14分42秒                                 │
│  申告者: 田中 浩 (****-****-1234)                                     │
│  申告区分: 令和8年度 確定申告書 (B様式)                                │
│                                                                       │
│  ┌─ 還付予定額 ────────────────────────────────────────────────────┐  │
│  │                                                                 │  │
│  │     ¥73,765                                                      │  │
│  │                                                                 │  │
│  │     還付予定日: 令和9年4月上旬                                  │  │
│  │     (約6週間後)                                                  │  │
│  │                                                                 │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  この受付通知は drive に保管されました。                                │
│  ファイル: personal-hiroshi-tanaka-jp/tax/fy2026/submission/           │
│           etax-receipt-20270227-1414-008-T-7842965.pdf                │
│  保管期間: 7年間 (令和16年2月27日まで)                                  │
│                                                                       │
│  お疲れさまでした、ひろしさん。                                        │
│  ("Good work, Hiroshi-san.")                                          │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

TalkBack: "受付完了しました。受付番号は2027-0227-1414-008-T-7842965です。還付予定額は7万3千7百65円です。還付予定日は令和9年4月上旬です。お疲れさまでした。"

Haptic: triple long pulse (celebration).

## Screen 8 — Diary entry (notes) voice dictation (14:36 JST)

```
┌──────────────────────────────────────────────────────────────────────┐
│  日記 — 晩年の記録 (late-life record keeping)                           │
│  ─────                                                                │
│                                                                       │
│  シリーズ: サチコへの年次手紙 (4年目)                                   │
│  日付: 令和9年2月27日                                                  │
│                                                                       │
│   ┌──────────────────────────────────────────────────────────────┐   │
│   │  〔録音中ボタン押下〕                                        │   │
│   │                                                              │   │
│   │  令和9年2月27日。確定申告を提出した。還付は7万3千7百65円。  │   │
│   │  タマは元気。みさきから明後日電話するそうだ。サチコへ —      │   │
│   │  今年もちゃんと終わらせました。あなたが見ていてくれている    │   │
│   │  といいな。                                                  │   │
│   │                                                              │   │
│   │  [読み返す]  [保存]  [修正]                                  │   │
│   │                                                              │   │
│   └──────────────────────────────────────────────────────────────┘   │
│                                                                       │
│  ⓘ この日記は晩年の記録として保管されます。                            │
│  ⓘ 相続準備が完了したら、家族 (みさき + だいき) に引き継がれます。    │
│  ("This diary is preserved as late-life record. When succession is    │
│   set up, it will be handed to your family (Misaki + Daiki).")        │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

UX notes:

- Voice dictation; TalkBack reads back the transcript before saving.
- Series tag is visible — Hiroshi can see this is the 4th year of letters to Sachiko.
- Succession-ready tag is visible — Hiroshi understands the diary will eventually transfer.
- No anxiety language about death — calm framing of "相続準備が完了したら" (when succession is set up).

## Accessibility invariants observed across all screens

| Invariant | Status |
|---|---|
| TalkBack 100% coverage of interactive elements | ✓ |
| Voice navigation supports core commands | ✓ |
| Voice recognition ≥ 92% confidence | ✓ 94% avg |
| High-contrast theme active | ✓ |
| Large-text body ≥ 18pt | ✓ 18pt min |
| Icon labels ≥ 24pt bold | ✓ |
| Haptic feedback on every action | ✓ |
| No red-only error indication | ✓ orange + voice slowdown |
| 30-second timeout on NFC retry | ✓ |
| No "error" or "failure" language | ✓ "もう一度どうぞ" used |
| Japanese full-width + Kanji preservation | ✓ NFC byte-exact |
