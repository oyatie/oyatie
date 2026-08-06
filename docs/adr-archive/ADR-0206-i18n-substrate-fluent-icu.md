---
id: ADR-0206
status: Superseded
deciders: council-architecture, axis-frontend, axis-product, axis-regional-pack
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-701]
related: [ADR-0064, ADR-0185, ADR-0204, ADR-0205, ADR-0207]
related_specs:
  - /specs/products/workflow-studio.json
---

# ADR-0206 — i18n substrate: Fluent (Mozilla) as source-of-truth + ICU MessageFormat surface

## Status

Accepted (2026-05-18). Mandates **Fluent (Mozilla)** as the canonical authoring format for translatable strings + **ICU MessageFormat** plural/select grammar across all client stacks. RTL bidi is locale-driven.

## Context

oyatie ships in multiple locales:

- en-US (source locale) + ko-KR + ja-JP + zh-Hans-CN + es-MX + pt-BR + de-DE + fr-FR + nl-NL + it-IT (Western Tier-1 priority);
- ar-SA + ar-AE + he-IL + fa-IR (RTL Tier-1);
- per-regional-pack overlays per ADR-0064 (KR pack first; UAE/SA pack next).

The bar:

- **Single source of truth** for translatable strings.
- **Rust-native** authoring (so Leptos / Rust services can consume).
- **Per-stack adapter** layer compiling source to platform-native catalog format.
- **Plural + gender + select** semantics (ICU MessageFormat grammar).
- **RTL support** for Arabic, Hebrew, Persian, Urdu, Pashto, Sindhi, Kurdish.
- **Per-tenant default locale** + per-user override + `Accept-Language` fallback.

Anti-patterns:

1. PO/MO (gettext) as source-of-truth — plural grammar is weak; less expressive than Fluent.
2. Per-stack catalog source (e.g., `.xcstrings` for Apple + `strings.xml` for Android + `.po` for web) — translators see N catalogs instead of one.
3. Hard-coded English strings in components with `// TODO: i18n` — debt accumulates fast.

## Decision

### Authoring source-of-truth: Fluent (Mozilla)

Translatable strings author at `clients/i18n/source.ftl` (Fluent grammar). Why Fluent over PO/MO:

- **Rust-native** — `fluent-rs` is the canonical Rust impl (maintained by Mozilla + community).
- **Expressive** — variants (gender, plural, select) + nested message references + terms (reusable noun phrases).
- **One source, many targets** — adapters compile Fluent → platform-native catalogs.

### Runtime surface: ICU MessageFormat

At rendering time, the message body uses ICU MessageFormat grammar for plural / gender / select. Fluent's pattern language is a superset of ICU MF for the cases we use; the adapter compiles Fluent variants to the target platform's ICU MF emitter.

### Per-stack adapter table

| Stack | Adapter | Catalog format |
|---|---|---|
| SvelteKit | `@formatjs/icu-messageformat-parser` + `svelte-i18n` (Fluent source compiled to JSON catalog) | per-locale `.json` |
| Leptos | `fluent-rs` + `leptos-fluent` | per-locale `.ftl` |
| SwiftUI (Apple) | Fluent → `String Catalog` (`.xcstrings`) generator | `.xcstrings` |
| Compose (Android) | Fluent → standard `strings.xml` with ICU plural support generator | `strings.xml` |
| GTK 4 (Linux) | Fluent → `gettext` generator | `.po` / `.mo` |
| WinUI 3 (Windows) | Fluent → `ResourceLoader` with ICU strings | `.resw` |

Generators live at `clients/i18n/gen-<stack>/` and run in CI when `source.ftl` changes.

### RTL bidi

Locales with RTL primary language subtag (`ar`, `he`, `fa`, `ur`, `ps`, `sd`, `ckb`, `ug`, `yi`) carry an `is_rtl=true` flag from the `oya-shared-i18n-kernel` `LocaleTag` type. UI layer reads the flag to switch `dir="rtl"` (web) / native bidi attribute (mobile / desktop) at the root layout.

### Locale routing

1. Per-tenant default locale (set in tenancy admin).
2. Per-user override (in user profile).
3. Per-request `Accept-Language` fallback.
4. Source locale (`en-US`) ultimate fallback.

### Coverage gate

`oya-check-i18n-coverage` (advisory) scans every µservice's `clients/i18n/source.ftl` plus locale overlays; fails advisory when a declared required locale has coverage below `min_coverage_bps` (manifest-tunable; default 9500 / 95%).

## Alternatives considered

### (a) gettext (PO/MO) as source-of-truth — REJECTED

- **Pros:** ubiquitous; every framework consumes PO.
- **Cons:** weak plural grammar (no select/gender); two-form pluralization only; AT&T-era format.
- **Rejected**: not expressive enough.

### (b) ICU MessageFormat 2.0 (MF2) as source-of-truth — DEFERRED

- **Pros:** modern; designed for the Web platform; cleaner than MF1.
- **Cons:** still Tech Preview as of late 2025; library support uneven.
- **Deferred**: revisit when MF2 reaches Recommendation status (CLDR-TC track).

### (c) i18next (JS-only) — REJECTED for source-of-truth

- **Pros:** widely adopted in JS land.
- **Cons:** JS-only ecosystem; doesn't serve Rust/Leptos/native stacks.
- **Rejected**: not cross-stack.

### (d) **CHOSEN: Fluent (Mozilla) source + ICU MessageFormat surface**

- **Pros:**
  - Fluent grammar expresses every variant case we need (plural, gender, select, term references).
  - `fluent-rs` is Rust-native; works in Leptos / kernel-level tooling.
  - Adapter generators compile to every target's idiomatic catalog format → translators see one source.
  - Mozilla operates Fluent at scale (Firefox + AMO + MDN).
- **Cons:** generators per stack require authoring. Mitigation: one-time cost; CI-gated.
- **Accepted**.

## Consequences

### Positive

1. **One source-of-truth.** Translators work in Fluent; downstream adapters compile.
2. **RTL-by-default.** Locale tag carries the bidi flag; UI flips at the root layout.
3. **Plural + select + gender** grammar via Fluent variants → ICU MF.
4. **Coverage gate** catches drift before merge.

### Negative

1. **Per-stack adapter generators** must be authored + maintained. Mitigation: shared CI matrix.
2. **Translators must learn Fluent.** Mitigation: Fluent grammar is small; tooling (Pontoon, Localazy) supports it.

### Operational

1. Source at `clients/i18n/source.ftl` per µservice (or top-level for cross-product strings).
2. Translator workflow: pull source → Pontoon-class translation memory → push per-locale `.ftl` overlays.
3. CI lane `oya-check-i18n-coverage` runs on every PR.

## In-house roadmap

**Vendor classification:** Fluent (Mozilla; community-maintained) + ICU MessageFormat (Unicode CLDR-TC; W3C track) are **community / standards** layers. No Phase 2 in-house rebuild.

- **No in-house rebuild planned.** Building a competing i18n authoring format would split translator tooling and forfeit the Pontoon / Localazy / Crowdin ecosystem.
- **What we DO build in-house:** per-stack Fluent → native-catalog generators at `clients/i18n/gen-<stack>/`, locale-routing middleware at `crates/oya-shared-i18n-kernel`, and the per-pack overlay convention (per ADR-0064).
- **Trigger conditions to revisit:** (i) Mozilla retires Fluent (extremely unlikely); (ii) MF2 reaches Recommendation status and adoption is broad enough to consider source migration.

## Rollback

- Adapter generator rollback: revert the generator + republish per-locale catalogs from the prior generator's output (cached in CI artifacts).
- Source-locale rollback: source.ftl is git-versioned; revert + regenerate.

## References

- Fluent (Mozilla) — https://projectfluent.org ; current spec stable as of 2026-05-18.
- `fluent-rs` — https://github.com/projectfluent/fluent-rs ; Rust-native.
- ICU MessageFormat — https://unicode-org.github.io/icu/userguide/format_parse/messages/ ; ICU 76+ stable.
- ICU MessageFormat 2.0 — https://github.com/unicode-org/message-format-wg ; Tech Preview as of late 2025.
- `@formatjs/icu-messageformat-parser` — https://formatjs.io
- `svelte-i18n` — https://github.com/kaisermann/svelte-i18n
- `leptos-fluent` — https://github.com/mondeja/leptos-fluent
- Apple String Catalog — https://developer.apple.com/documentation/xcode/localization
- ADR-0064 — canonical base + localization (pack model).
- ADR-0185 — Workflow Studio client stack.
- ADR-0207 — a11y bar (RTL bidi tied to a11y).
- LTS-rotation cadence: versions current as of 2026-05-18; review per ADR-0098.
