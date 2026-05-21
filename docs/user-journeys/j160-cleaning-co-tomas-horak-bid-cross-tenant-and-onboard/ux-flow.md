---
doc_class: User-Journey-UX-Flow
journey_id: j160-cleaning-co-tomas-horak-bid-cross-tenant-and-onboard
date: 2026-05-20
authority_tier: 2
status: draft
---

# j160 — UX flow: 81-day cross-tenant bid → onboard cascade

Five device contexts: Tomáš's Lenovo Tab P12 Pro at the Skvrňany depot (Czech-primary; touch-keyboard with diacritic-strict mode); Tomáš's iPhone 13 mini on-site at PolyCraft; Procházková's Dell Latitude 5340 at PolyCraft (work-tenant UI, Czech-primary, English-secondary); foreman Pavel Novák's Samsung A35 tablet on first-shift morning; Datová schránka mobile screens (Czech-state-mandated UI surface).

The unifying UX rule: the **tenant chip + diacritic-strict mode chip** persist at the top of every screen. Czech is primary locale; English secondary fallback only.

## Screen 1 — Bid request render in marketplace (Tuesday Oct 14 14:42 CET · Tomáš's Lenovo Tab P12 Pro)

```
┌──────────────────────────────────────────────────┐
│ 🏢 uklid-horak-sro-plzen-cz · firma             │
│ čeština · NFC diacritic-strict                  │
├──────────────────────────────────────────────────┤
│                                                  │
│  🏛 Otevřená výzva k podání nabídky              │
│                                                  │
│  Zadavatel: PolyCraft Bohemia a.s.               │
│             IČ 47714232 · DIČ CZ47714232         │
│             Plzeňský závod                       │
│                                                  │
│  Předmět: úklid průmyslového areálu              │
│           24 měsíců                              │
│           start: 2027-01-04 06:00 CET            │
│           konec: 2028-12-31 23:59 CET            │
│                                                  │
│  Plocha celkem: 12 400 m²                        │
│  ├ výrobní:       9 200 m²                       │
│  ├ sklad:         1 800 m²                       │
│  ├ administrativa:  980 m²                       │
│  └ společné:        420 m²                       │
│                                                  │
│  Maximální cena bez DPH (ročně):                 │
│  CZK 4 200 000 / rok                             │
│  CZK 8 400 000 / 24 měsíců                       │
│                                                  │
│  Požadavky:                                      │
│  ✓ ČSN-EN-13549 minimální stupeň 4               │
│  ✓ ISO 9001 certifikát platný                    │
│  ✓ ISSA-CIMS doporučeno                          │
│  ✓ Pojištění odpovědnosti min. CZK 50M           │
│                                                  │
│  Termín pro podání nabídek:                      │
│  📅 pátek 17. 10. 2026 17:00 CET                 │
│                                                  │
│  Konkurence: 4 další účastníci (anonymně)        │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │  📋 ZAHÁJIT PŘÍPRAVU NABÍDKY            │    │
│  └─────────────────────────────────────────┘    │
└──────────────────────────────────────────────────┘
```

UX notes:

- The bid is rendered fully in Czech with proper diacritics throughout — including "Plzeňský závod" and "Maximální cena bez DPH".
- The "Konkurence: 4 další účastníci (anonymně)" line gives Tomáš situational awareness without revealing identities.
- The single primary action "ZAHÁJIT PŘÍPRAVU NABÍDKY" (initiate bid prep) opens the workflow.

## Screen 2 — Site walk evidence capture (Wednesday Oct 15 09:18 CET · Lenovo Tab in production-floor zone 3)

```
┌──────────────────────────────────────────────────┐
│ 🏢 uklid-horak-sro-plzen-cz                      │
├──────────────────────────────────────────────────┤
│ TASK 1 · obhlídka závodu PolyCraft               │
│         site walk evidence capture                │
│                                                  │
│   📷 capture                                     │
│   ┌───────────────────────────┐                  │
│   │  [live camera view]       │                  │
│   │                           │                  │
│   │  Zóna: výrobní 3 (com-    │                  │
│   │        pounding)          │                  │
│   │  Pozice na podlaze:       │                  │
│   │     residue pattern       │                  │
│   └───────────────────────────┘                  │
│                                                  │
│   Typ záznamu:                                   │
│   ◯ Zóna celkem                                  │
│   ◯ Stav podlahy                                 │
│   ● Reziduum rozpouštědla ← zvoleno              │
│   ◯ Vybavení                                     │
│   ◯ Bezpečnostní problém                         │
│                                                  │
│   Hlasová poznámka (CS):                         │
│   🎤 [00:24/02:00]                               │
│   "Tady je ten problém. Použít neutralizér +     │
│   low-residue surfactant + Tennant T7AMR s       │
│   mikrofiber padem..."                           │
│                                                  │
│   foto: 8 / 11                                   │
│                                                  │
│   ┌─────────────────────────────────────────┐   │
│   │   📷 ULOŽIT FOTO                        │   │
│   └─────────────────────────────────────────┘   │
└──────────────────────────────────────────────────┘
```

UX notes:

- Voice notes in Czech auto-transcribe; diacritics preserved at byte level.
- Photo classification at capture time avoids later metadata gymnastics.
- The zone identifier auto-fills from Procházková's pre-shared zone map.

## Screen 3 — Bid line items structured form (Wed Oct 15 16:18 CET · Lenovo Tab in depot)

```
┌──────────────────────────────────────────────────┐
│ 🏢 uklid-horak-sro-plzen-cz                      │
├──────────────────────────────────────────────────┤
│ Nabídka · cenová struktura · ČSN-EN-13549        │
│                                                  │
│ ┌─ Zóna ───────┬─m²──┬ Freq ─┬ Grade ┬ CZK/měs ┐│
│ │ Výroba z.1   │1400 │ N+M   │   4   │ 49 400  ││
│ │ Výroba z.2   │1800 │ N+W   │   4   │ 62 200  ││
│ │ Výroba z.3   │2200 │ N+BW  │   4   │ 66 400  ││
│ │ Výroba z.4-7 │3800 │ N+M   │   4   │ 85 000  ││
│ │ Sklad        │1800 │ N     │   3   │ 19 000  ││
│ │ Administ.    │ 980 │ 5×T   │   4   │ 22 600  ││
│ │ Kantýna      │ 180 │ 5+M   │   4   │ 10 200  ││
│ │ Šatny+sprchy │ 180 │ N+M   │   4   │  7 200  ││
│ │ WC (8 sad)   │ 480 │ 5×T   │   4   │ 10 800  ││
│ ├──────────────┼─────┼───────┼───────┼─────────┤│
│ │ MĚSÍČNĚ      │     │       │       │ 332 800 ││
│ │ ZA 24 MĚSÍCŮ │     │       │       │7 940 000││
│ │ + DPH 21%    │     │       │       │1 667 400││
│ │ CELKEM       │     │       │       │9 607 400││
│ └──────────────┴─────┴───────┴───────┴─────────┘│
│                                                  │
│ Příloha protokolu ČSN-EN-13549: ✓ připojeno      │
│ ISO 9001 cert: ✓ připojeno                       │
│ ISSA-CIMS cert: ✓ připojeno                      │
│ Pojištění Generali: ✓ připojeno                  │
│ Reference Plzeňský Prazdroj: ✓ připojeno         │
│ Reference Škoda Auto Plzeň: ✓ připojeno          │
│ Plán složení posádky: ✓ připojeno                │
│ Průvodní dopis: ✓ připojeno                      │
│                                                  │
│ Diacritic check: Plzeňský ✓ Šimková ✓ Hoàng ✓    │
│                                                  │
│ ┌─────────────────────────────────────────────┐ │
│ │  📨 PODAT NABÍDKU · 7 940 000 CZK bez DPH    │ │
│ └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

UX notes:

- The structured cost table is the heart of the screen; freq codes (N=nightly, M=monthly deep, W=weekly deep, BW=biweekly deep, 5×T=5×weekday) are localized but learnable.
- The "Diacritic check" line confirms the system handled Tomáš's planned crew names correctly.
- The single primary action shows the bid total in big bold CZK.

## Screen 4 — Award notification (Mon Oct 27 14:00 CET · Tomáš's iPhone 13 mini)

```
┌──────────────────────────────────────────────────┐
│ 🏢 uklid-horak-sro-plzen-cz                      │
├──────────────────────────────────────────────────┤
│                                                  │
│       🏆 ROZHODNUTÍ O VÝBĚRU NABÍDKY             │
│                                                  │
│  PolyCraft Bohemia a.s. → Úklid Horák s.r.o.     │
│                                                  │
│  Zakázka: úklid Plzeňského závodu                │
│          2027-01-04 → 2028-12-31                 │
│          24 měsíců                               │
│                                                  │
│  Vaše nabídka:                                   │
│  CZK 7 940 000 bez DPH                           │
│  CZK 9 607 400 s DPH                             │
│                                                  │
│  Rozhodnutí: ✓ AKCEPTOVÁNO                       │
│                                                  │
│  Vyhodnotila:                                    │
│  Ing. Martina Procházková                        │
│  vedoucí nákupu                                  │
│  PolyCraft Bohemia a.s.                          │
│                                                  │
│  14:00:12 CET, 27. 10. 2026                      │
│                                                  │
│  Audit dual-seal: ✓ ověřeno                      │
│                                                  │
│  Další krok:                                     │
│  📝 vyjednání smlouvy → start 28. 10. 09:00      │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │   📨 OPEN MESSENGER · M. PROCHÁZKOVÁ    │    │
│  └─────────────────────────────────────────┘    │
└──────────────────────────────────────────────────┘
```

UX notes:

- The award screen is celebratory but precise — no marketing flourish.
- The "Audit dual-seal: ✓ ověřeno" line gives Tomáš provable confidence in the award.
- The next-step prompt makes the contract-negotiation phase explicit.

## Screen 5 — Contract QES dual-tenant signing (Fri Nov 7 11:18 CET · Lenovo Tab P12)

```
┌──────────────────────────────────────────────────┐
│ 🏢 uklid-horak-sro-plzen-cz                      │
├──────────────────────────────────────────────────┤
│ Smlouva o úklidových službách                    │
│ Úklid Horák s.r.o. ↔ PolyCraft Bohemia a.s.      │
│ kontrakt: contract-uklid-horak-polycraft-2027    │
│                                                  │
│ Hodnota: CZK 7 940 000 bez DPH                   │
│ Doba: 24 měsíců, 2027-01-04 → 2028-12-31         │
│                                                  │
│ Stránek: 38 (3 hlavní + 5 příloh A-E)            │
│ Hash kontraktu (SHA-256):                        │
│ 8f4a2c91d7e6b3a5...                              │
│                                                  │
│ ─── PODPIS ZA UKLID HORAK S.R.O. ───             │
│ Podepisující: Tomáš Horák, jednatel              │
│ QES poskytovatel: I.CA                           │
│ Certifikát platný do: 2027-09-12                 │
│                                                  │
│ TrueTime fence:                                  │
│ • uncertainty: 6 ms ≤ 10 ms ✓                    │
│                                                  │
│ Po podpisu:                                      │
│ • Datová schránka notifikace                     │
│ • Audit dual-seal v obou tenantech               │
│ • Workflow → contract_signed                     │
│                                                  │
│ Diacritic check:                                 │
│ "Úklid Horák s.r.o." ✓                          │
│ "Tomáš Horák" ✓                                  │
│ "Martina Procházková" ✓                          │
│                                                  │
│ ┌─────────────────────────────────────────────┐ │
│ │  ✕ ZRUŠIT │ ✓ PODEPSAT KVALIFIKOVANĚ        │ │
│ └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

UX notes:

- QES (qualified electronic signature) provider is shown (I.CA is the Czech state-recognized CA most commonly used).
- TrueTime uncertainty in milliseconds is shown — a Cedar invariant.
- The contract hash is shown so Tomáš can verify integrity with the printed copy if he wants.
- Diacritic verification is on-screen as a small but critical signal.

## Screen 6 — OSSZ employee registration (Mon Nov 30 11:42 CET · Lenovo Tab P12 in depot)

```
┌──────────────────────────────────────────────────┐
│ 🏢 uklid-horak-sro-plzen-cz                      │
│ → cross-tenant: cz-ossz-state-tenant             │
├──────────────────────────────────────────────────┤
│ OSSZ · Registrace zaměstnance                    │
│                                                  │
│ Zaměstnavatel:                                   │
│   Úklid Horák s.r.o.                             │
│   IČ 27488123 · DIČ CZ27488123                   │
│                                                  │
│ Nový zaměstnanec:                                │
│   Jméno: Hoàng Văn Long                          │
│          (Vietnamese tones preserved ✓)          │
│   Datum narození: 14. 3. 1992                    │
│   Rodné číslo: 920314/XXXX                       │
│   Bydliště: Plzeň-Skvrňany, ...                  │
│   Status pobytu: trvalý pobyt                    │
│   Druh smlouvy: doba neurčitá, HPP               │
│   Datum nástupu: 1. 12. 2026                     │
│                                                  │
│ ─── kontrola diakritiky ───                      │
│ "Hoàng Văn Long" — 4 diakritické znaky:          │
│   o + à grave  ✓                                 │
│   ă breve     ✓                                  │
│   n + tilde-equivalent (in tone) ✓               │
│ Žádná normalizace na ASCII ✓                     │
│                                                  │
│ ─── tax-pripravy ───                             │
│ Daň z příjmu FO: nastaveno (Finanční úřad TR)   │
│ Zdravotní pojišťovna: VZP (zaměstnanec volil)    │
│                                                  │
│ ┌─────────────────────────────────────────────┐ │
│ │  📨 ODESLAT NA OSSZ                          │ │
│ └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

UX notes:

- The Vietnamese tone marks in "Hoàng Văn Long" are shown as preserved at byte level — important for a Vietnamese-Czech employee who has previously experienced his name being stripped.
- ARES + Finanční úřad + ZP integrations are shown as pre-completed (auto-population from the unified Czech state systems context).
- One-tap to OSSZ; the receiving system is Czech-state mandated.

## Screen 7 — Biometric badge enrollment (Wed Dec 17 14:18 CET · cross-tenant onboarding station at PolyCraft)

```
┌──────────────────────────────────────────────────┐
│ 🏢 uklid-horak-sro-plzen-cz                      │
│ → cross-tenant: polycraft-bohemia-as-plzen-cz   │
├──────────────────────────────────────────────────┤
│ Biometrická registrace na přístupový systém      │
│ PolyCraft Plzeň · ISO 27001 access control       │
│                                                  │
│ Zaměstnanec:                                     │
│   Pavel Novák · vedoucí čety                     │
│                                                  │
│ Kontext:                                         │
│   kontrakt: contract-uklid-horak-polycraft-2027  │
│   role: cleaner-team-lead                        │
│                                                  │
│ Cedar předpoklady:                               │
│ ✓ ČSN-262-2006 školení dokončeno (2026-12-05)    │
│ ✓ GDPR + CZ-110/2019 školení dokončeno (2026-12-10)│
│ ✓ PolyCraft indukce dokončeno (2026-12-17)       │
│                                                  │
│ Biometric template:                              │
│   typ: fingerprint minutiae ISO 19794-2          │
│   palec dominantní ruky                          │
│   hash uložen: ✓                                 │
│                                                  │
│ Scope:                                           │
│ ✓ Access PolyCraft Plzeň plant entry points      │
│ ✓ Production zone 1-7 entry                      │
│ ✓ Cleaner-storage room                           │
│ ✗ Office floors (admin) — limited               │
│ ✗ Other PolyCraft sites (MB, Ostrava) — denied  │
│                                                  │
│ Auto-revoke: konec kontraktu 2028-12-31          │
│              nebo dříve při ukončení             │
│                                                  │
│ ┌─────────────────────────────────────────────┐ │
│ │  ✕ ZRUŠIT │ ✓ ZAREGISTROVAT                  │ │
│ └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

UX notes:

- The Cedar prerequisites are listed; only meeting all three unlocks enrollment.
- The biometric scope is explicitly limited (no Mladá Boleslav or Ostrava sites — only Plzeň).
- Auto-revoke at contract end is the doctrine; ambient retention is forbidden.

## Screen 8 — Datová schránka notification (Fri Nov 7 11:42 CET · Tomáš's iPhone, redirected to mojedatovaschranka app)

```
┌──────────────────────────────────────────────────┐
│ 📮 Datová schránka                               │
│    moje datová schránka mobile                  │
├──────────────────────────────────────────────────┤
│                                                  │
│  Nová zpráva (zaknihování smlouvy)               │
│                                                  │
│  Odesílatel: oyatie audit-chain                  │
│             (zastoupení provozovatele)           │
│  IČ vlastníka: 27488123 (Úklid Horák s.r.o.)     │
│  IČ protistrany: 47714232 (PolyCraft Bohemia)    │
│                                                  │
│  Předmět: zaknihování zakázky                    │
│           contract-uklid-horak-polycraft-2027    │
│                                                  │
│  Hodnota: CZK 7 940 000 bez DPH                  │
│  Hash kontraktu (SHA-256):                       │
│  8f4a2c91d7e6b3a5...                             │
│  Podpis dual-tenant QES: ✓                       │
│  Čas podpisu: 7. 11. 2026 11:18:18 CET           │
│                                                  │
│  Archivační lhůta: 7 let (CZ-Civil-Code)         │
│                                                  │
│  Zpráva má právní účinky od:                     │
│  📅 dodání do datové schránky                    │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │  📥 OTEVŘÍT                              │    │
│  └─────────────────────────────────────────┘    │
└──────────────────────────────────────────────────┘
```

UX notes:

- The Datová schránka system uses its own native UI (not oyatie's); oyatie hands off via the legally-recognized integration point.
- The Datová schránka mailbox is Czech-state-mandated; messages here have legal effect equivalent to registered mail.

## Screen 9 — First shift gate scan (Mon Jan 4 2027 05:48 CET · Pavel's Samsung A35 tablet at PolyCraft security gate)

```
┌──────────────────────────────────────────────────┐
│ 🏢 polycraft-bohemia-as-plzen-cz                 │
│ guard station view                              │
├──────────────────────────────────────────────────┤
│                                                  │
│  Příchod posádky · Úklid Horák s.r.o.            │
│  smlouva: contract-uklid-horak-polycraft-2027    │
│                                                  │
│  ✓ Pavel Novák           05:48:12 fingerprint OK │
│  ✓ Lenka Šimková         05:48:38 fingerprint OK │
│  ✓ Hoàng Văn Long        05:49:02 fingerprint OK │
│  ✓ Mária Kováčová        05:49:24 fingerprint OK │
│  ✓ Іван Шевченко         05:49:48 fingerprint OK │
│                                                  │
│  Všech 5 zaregistrováno · gate green             │
│                                                  │
│  Audit dual-seal:                                │
│  • polycraft-bohemia-as-plzen-cz ✓               │
│  • uklid-horak-sro-plzen-cz ✓                    │
│                                                  │
│  První směna: 06:00 → 14:00                     │
│  Zóny: výroba 1-3 (Pavel + Lenka)                │
│         výroba 4-7 (Hoàng + Mária)               │
│         sklad + admin (Іван)                     │
│                                                  │
│  Tomáš Horák je na místě jako pozorovatel        │
│                                                  │
└──────────────────────────────────────────────────┘
```

UX notes:

- Every name preserves its native diacritics — including "Іван Шевченко" in Cyrillic.
- Both tenants' audit dual-seal checkmarks are visible to the guard, indicating audit-chain integrity.
- Tomáš's owner-on-site role is acknowledged.

## Screen 10 — Czech cleaning-industry community post (Thu Nov 13 19:42 CET · Lenovo Tab in Tomáš's home study)

```
┌──────────────────────────────────────────────────┐
│ 🏢 uklid-horak-sro-plzen-cz                      │
│ ↓ posting into:                                  │
│ 👥 cz-cleaning-industry-owner-operators-community│
│    community · 184 members · Czech-language      │
├──────────────────────────────────────────────────┤
│                                                  │
│  📝 Nový příspěvek · otázka                      │
│                                                  │
│  [Mám otázku ohledně přijímání ukrajinského      │
│   uprchlíka s dočasnou ochranou. Jaké zvláštní   │
│   GDPR-nuance se týkají special category data    │
│   pro tyto zaměstnance? Konkrétně: facility-     │
│   access records klienta budou obsahovat osobní  │
│   údaje včetně biometrického hashe. Klient       │
│   přechází na nás jako data processor; my máme   │
│   na sub-data-processor pozici i zaměstnance     │
│   samotného? Nějaké zkušenosti z praxe?]         │
│                                                  │
│  Tagy: gdpr · cz-110-2019 · ukrajinští-uprchlíci │
│       · biometrika · special-category-data       │
│                                                  │
│  ⚠ Tento příspěvek žije POUZE v komunitním       │
│    tenantu. Tvůj klientský tenant (PolyCraft)    │
│    nemá VIDITELNOST.                            │
│    Komunita má 184 ověřených členů.             │
│    MLS-encrypted; epoch 122.                     │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │   📨 ZVEŘEJNIT                          │    │
│  └─────────────────────────────────────────┘    │
└──────────────────────────────────────────────────┘
```

UX notes:

- The dual chip (primary tenant + community tenant) makes clear this post lives in the third tenant.
- The Ukrainian-refugee question is sensitive; the community tenant offers the right peer-confidentiality context.
- The "Tvůj klientský tenant nemá VIDITELNOST" reassurance is critical for blue-collar owner-operators who fear they will accidentally leak operational concerns to their clients.

## Locale + accessibility

- Tomáš's locale: `cs-CZ` primary; `de-DE` secondary (B2); `en-GB` tertiary (B1)
- Procházková's locale: `cs-CZ` primary; `en-GB` secondary
- Czech diacritic input: native physical keyboard preferred; touch IME supports diacritic-strict mode
- Vietnamese-tone input for Hoàng Văn Long's payslip and self-service portal: full IME
- Cyrillic input for Іван Шевченко's self-service portal: full IME
- Font: Czech-Latin extended + Vietnamese diacritics + Slovak + Cyrillic all rendered in single typographic stack
- Color tokens: business-tenant chip muted-blue (#2A6F97); state-tenant chip slate (#4A5568); community-tenant chip warm-amber (#D9822B)
- Accessibility: WCAG AAA contrast for tenant chips; VoiceOver Czech reads tenant name + diacritic-strict mode
- Voice fallback: Czech voice input fully supported (Tomáš uses voice notes in pressroom regularly)
- Guard station screens: high-contrast for sub-optimal lighting; 6:1 contrast minimum

## Failure-mode UX

| Failure | UX response |
|---|---|
| Diacritic normalization attempted on legal field (Tomáš → Tomas) | Hard error; field write rejected; user shown the offending field with diff |
| Bid submit after window closes | Hard refusal with countdown showing how late; alternate path: contact issuing tenant for grace |
| QES certificate expired | Pre-flight warning 30 days out; alternate-CA fallback; Cedar deny on actual sign |
| TrueTime fence breach (uncertainty >10 ms) | Sign refused; retry with backoff; manual escalation path if persistent |
| Datová schránka unreachable | Notification queued locally; retry; legal-effect timer paused |
| Biometric template enrollment without training prerequisites | Cedar deny; UI shows which training is missing |
| OSSZ system rejection (invalid rodné číslo format) | Error returned in Czech with field highlighted; alternate input methods offered |
| Cross-tenant ARES verification mismatch (IČ vs DIČ inconsistency) | Hard error; fix via ARES; cannot proceed with bid |

## Stop condition

The UX flow is correct when Tomáš can complete the 81-day journey in Czech-primary locale with all diacritic-bearing fields persisted at byte-level fidelity, when QES dual-tenant signing under TrueTime fence works without ambiguity, when Czech state-system integrations (ARES + OSSZ + ZP + Finanční úřad + Datová schránka) are presented as first-class concerns within the Tomáš-visible flow, when cross-tenant biometric badge enrollment is Cedar-scoped rather than ambient, and when the first-shift gate scan dual-seals in both tenants at the contract-live boundary.
