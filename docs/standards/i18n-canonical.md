---
doc_class: Standard
shape: standard
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-18
purpose: |
  Canonical i18n authoring + locale-routing standard. Sources of truth, grammar, plural/select rules,
  per-stack catalog generators, and required test recipes. Companion: a11y-canonical.md, rtl-rendering.md,
  locale-routing.md, wcag-2.2-aa-checklist.md.
canonical_authority: docs/decisions/ADR-0701-monorepo-capability-live-apex.md
related_adrs:
  - ADR-0064
  - ADR-0185
  - ADR-0206
  - ADR-0207
enforced_by: oya-check-i18n-coverage
---

# i18n Canonical Standard

## Authority

This standard implements ADR-0206. Every user-visible string in oyatie is authored in **Fluent (Mozilla)**
at `clients/i18n/source.ftl` and compiled per-stack via the adapter generators at `clients/i18n/gen-<stack>/`.
Runtime plural / gender / select grammar follows **ICU MessageFormat**.

## Authoring rules (RFC-2119)

1. Every user-visible string **MUST** carry a stable `MessageId` (Fluent identifier grammar:
   leading ASCII letter, then letters / digits / `-` / `_`).
2. Translatable strings **MUST NOT** be inlined as string literals in component source.
   Components reference messages by id.
3. The source locale is `en-US`. All variant catalogs **MUST** track the source by `MessageId`.
4. Per-microservice catalogs live at `microservices/<ms>/clients/i18n/source.ftl`. Cross-product
   catalogs live at `clients/i18n/source.ftl`.
5. Per-regional pack overlays (per ADR-0064) live at
   `microservices/<ms>/clients/i18n/packs/<pack>/<locale>.ftl`.
6. **No string concatenation across i18n boundaries.** Use Fluent variants + ICU select.
7. **Plural** uses ICU plural categories (`zero`, `one`, `two`, `few`, `many`, `other`); locale-driven.

## Fluent example

```ftl
# Workflow Studio canvas title
workflow-studio-canvas-title = Workflow Studio
workflow-studio-canvas-node-count =
    { $count ->
        [0] No nodes yet
        [one] 1 node
       *[other] { $count } nodes
    }
```

## Coverage gate

`oya-check-i18n-coverage` (advisory) computes per-locale coverage in basis-points (10000 = 100%).
Threshold defaults to **9500 bps (95%)** for production-promoted locales; **9000 bps (90%)**
for tier-2 locales; **8000 bps (80%)** for beta locales. Per-µservice manifest overrides.

## Per-stack adapter table

| Stack | Adapter | Target catalog |
|---|---|---|
| SvelteKit | `@formatjs/icu-messageformat-parser` + `svelte-i18n` | per-locale `.json` |
| Leptos | `fluent-rs` + `leptos-fluent` | per-locale `.ftl` |
| SwiftUI (Apple) | Fluent → `String Catalog` generator | `.xcstrings` |
| Compose (Android) | Fluent → `strings.xml` with ICU plurals | `strings.xml` |
| GTK 4 (Linux) | Fluent → gettext generator | `.po` / `.mo` |
| WinUI 3 (Windows) | Fluent → `ResourceLoader` generator | `.resw` |

## Translation workflow

1. Author authors English source in `source.ftl`.
2. PR merges → CI generates per-stack catalogs into `clients/i18n/gen-<stack>/<locale>/`.
3. Translation memory tool (Pontoon / Localazy / Crowdin) pulls source; pushes per-locale overlays.
4. Per-locale overlays land as `<locale>.ftl` files; CI regenerates per-stack catalogs.
5. Coverage gate confirms ≥ threshold for required locales.

## Cross-references

- ADR-0206 — i18n substrate (Fluent + ICU).
- ADR-0064 — canonical base + localization (pack model).
- `rtl-rendering.md` — RTL bidi rules.
- `locale-routing.md` — per-tenant + per-user + Accept-Language routing.
- `a11y-canonical.md` — a11y interaction with i18n.
