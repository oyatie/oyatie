---
doc_class: Standard
shape: standard
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-18
purpose: |
  Canonical RTL (right-to-left) rendering rules. Locale-driven bidi flag from
  shared-i18n-kernel determines per-surface direction.
canonical_authority: docs/decisions/ADR-0701-monorepo-capability-live-apex.md
related_adrs:
  - ADR-0064
  - ADR-0206
  - ADR-0207
---

# RTL Rendering Standard

## Authority

This standard implements the RTL bidi rules called out in ADR-0206. Locale tags carry an
`is_rtl` flag (closed CLDR allowlist) via the `shared-i18n-kernel::LocaleTag` type.

## RTL locales (closed allowlist)

Primary language subtags considered RTL:

`ar` (Arabic), `he` (Hebrew), `fa` (Persian / Farsi), `ur` (Urdu), `ps` (Pashto),
`sd` (Sindhi), `ckb` (Central Kurdish / Sorani), `ug` (Uyghur), `yi` (Yiddish).

## Rendering rules (RFC-2119)

1. Web surfaces **MUST** set `dir="rtl"` on `<html>` when locale `is_rtl` is true.
2. CSS logical properties **MUST** be used (`margin-inline-start` instead of `margin-left`).
3. Icons with direction (← back, → forward, send) **MUST** flip when `is_rtl` is true.
4. Numbers + ISO 8601 dates remain LTR even in RTL surfaces (per Unicode Bidi Algorithm).
5. Mixed-script content (LTR English brand in RTL Arabic page) **MUST** use the Unicode
   Bidi Algorithm with explicit `‫` (RLE) / `‪` (LRE) marks ONLY when the
   algorithm produces wrong rendering.
6. Forms: input fields with email / URL types remain LTR (browser default).
7. Tooltip + dropdown positioning **MUST** mirror (right anchor → left anchor in RTL).
8. Canvas (ADR-0204): coordinate origin stays at top-left; node labels respect locale bidi.

## Per-stack RTL adapter table

| Stack | Direction primitive |
|---|---|
| SvelteKit | `<html dir="rtl">` + CSS logical properties |
| Leptos | `<html dir="rtl">` + CSS logical properties |
| SwiftUI | `.environment(\.layoutDirection, .rightToLeft)` |
| Compose | `LayoutDirection.Rtl` via `CompositionLocalProvider` |
| GTK 4 | `gtk_widget_set_direction(widget, GTK_TEXT_DIR_RTL)` |
| WinUI 3 | `FlowDirection.RightToLeft` |

## Testing

- Web: Playwright + axe-core runs each test once per direction (`ltr` + `rtl`).
- Native: per-platform UI test framework asserts mirrored layout.
- Visual regression: per-locale screenshot diff (LTR baseline + RTL baseline).

## Anti-patterns

1. Hard-coded `text-align: left` — use `text-align: start`.
2. Hard-coded `margin-left` — use `margin-inline-start`.
3. Per-component conditional `if (isRtl)` — bidi must be driven at the root.
4. Translating dates / numbers via string manipulation — use `Intl.DateTimeFormat` / `Intl.NumberFormat`.

## Cross-references

- ADR-0206 — i18n substrate (RTL flag source).
- ADR-0064 — per-pack overlays (KR pack LTR; UAE/SA pack RTL).
- `i18n-canonical.md` — Fluent authoring.
- Unicode Bidirectional Algorithm — https://www.unicode.org/reports/tr9/
