# Emoji + Sticker + Reaction + GIF System Architecture

**Document type**: Cross-microservice standards / architecture specification
**Lane**: standards
**Status**: DRAFT v1.0 — production-grade specification (intern-buildable)
**Authors**: oyatie messaging + UX + marketplace working group
**Last updated**: 2026-05-20
**Supersedes**: (none — first canonical draft)
**Related ADRs**: ADR-0145 (inter-microservice communication reform), ADR-0243 (Cedar as universal gate), ADR-0249 (marketplace substrate), ADR-0251 (compliance pack + cell certification levels)
**Cross-microservice owners**: `messenger`, `community`, `mail`, `docs`, `drive`, `slides`, `sheets`, `tasks`, `social`, `marketplace`, `plugin-app-store`

> Universal expression substrate. Wherever a user can type, they can react, sticker, GIF, and custom-emoji. One spec, one set of pixels, one set of policies, every surface.

---

## Table of contents

1. [Purpose + Scope](#1-purpose--scope)
2. [Unicode Emoji](#2-unicode-emoji)
3. [Sticker System](#3-sticker-system)
4. [Reactions](#4-reactions)
5. [GIF System](#5-gif-system)
6. [Custom Emoji + Sticker Creation](#6-custom-emoji--sticker-creation)
7. [Storage](#7-storage)
8. [Cross-platform Rendering](#8-cross-platform-rendering)
9. [Performance](#9-performance)
10. [Accessibility](#10-accessibility)
11. [Localization](#11-localization)
12. [Moderation](#12-moderation)
13. [Compliance](#13-compliance)
14. [Marketplace + Monetization](#14-marketplace--monetization)
15. [Lottie Animation Support](#15-lottie-animation-support)
16. [Bounded Contexts](#16-bounded-contexts)
17. [User Stories](#17-user-stories)
18. [References](#18-references)
19. [Appendices](#19-appendices)

---

## 1. Purpose + Scope

### 1.1 Mission statement

The Emoji + Sticker + Reaction + GIF System (hereafter ESRG) is the canonical, cross-surface, cross-tenant, cross-platform expression substrate of oyatie. Every user-facing surface that allows authored text — Messenger DMs, Messenger channels, Community posts, Mail compose, Docs comments, Drive comments, Slides comments, Sheets cell comments, Tasks comments, Social posts, Forms responses (where opted-in), and the Foundry runner UI — uses ESRG for expressive payloads.

A user who learns the sticker picker in Messenger should find the same picker, the same shortcuts, the same packs, the same custom emoji, and the same accessibility behavior in Docs. Pack purchases follow the user; pack policies follow the tenant; pack contents follow Cedar.

### 1.2 Benchmark + parity targets

ESRG is benchmarked against best-in-class consumer + enterprise messaging systems:

| Product | Strength we match | Strength we exceed |
|---|---|---|
| Discord | Custom server emojis; per-server emoji limits; super-reactions | Cross-surface (not just messenger); Cedar-gated install; tenant policy |
| Slack | Reaction-centric UX; custom emoji upload; reaction picker | Animated reactions; super-reactions; marketplace; cross-surface |
| KakaoTalk (KR) | Sticker store; LINE-class animation quality; KR-locale defaults | Per-tenant install; air-gapped tenant option |
| LINE (JP) | Sticker shop; creator economy | Cedar-gated; marketplace creator revenue share; multi-locale |
| WhatsApp | WebP sticker pack format; reaction simplicity | Cross-surface; tenant policy; super-reactions |
| iMessage | Sticker drawer + memoji | Cross-platform (not Apple-only); cross-tenant policy |
| Telegram | Sticker + animated sticker (.tgs Lottie-based) format | dotLottie 2024 spec; per-tenant pack registry; marketplace |
| Microsoft Teams | Reaction + GIF; tenant-policy controls | Marketplace; custom emoji + per-server packs |

> The product question this document answers is not "which features do we ship?" but "how do all of them coexist coherently, cross-surface, cross-tenant, cross-platform, cross-jurisdiction, without bloating any single microservice's boundary?"

### 1.3 Non-goals

- **Direct hosting of every sticker creator's pack** — we proxy via the marketplace; we do not run a creator-portal CMS outside the marketplace bounded context. (Marketplace lives in `microservices/marketplace`; sticker IP licensing lives there per ADR-0249.)
- **First-party rendering of complex 3-D animated emoji** (e.g., Memoji 3-D head tracking) — out of scope until v2.
- **Sticker-based game payments** (e.g., LINE Pay sticker minigames) — explicitly excluded; this substrate is expressive, not transactional.
- **Direct integration with third-party closed sticker stores** (LINE, KakaoTalk official packs) — those are walled-garden; we ship our own marketplace + allow tenant-uploaded compatibility imports.
- **AI-generated stickers** — explicitly v2 scope (will route via Intelligence Substrate per a future ADR; tracked in roadmap section).

### 1.4 In-scope surfaces

A surface uses ESRG by importing the `oya-expression-*` client SDK and Cedar-gating sticker/pack/GIF operations. The following surfaces are first-class in v1:

- `microservices/messenger` (DM + channel + thread + huddle chat overlay)
- `microservices/community` (posts + comments + reactions)
- `microservices/mail` (inline emoji + reaction-to-thread)
- `microservices/docs` (inline comments + suggestions)
- `microservices/drive` (file comments)
- `microservices/sheets` (cell comments)
- `microservices/slides` (slide comments)
- `microservices/tasks` (task comments + reactions)
- `microservices/social` (post reactions + comment reactions)
- `microservices/notes` (note inline + reactions)
- `microservices/forms` (opt-in for response forms; Cedar-gated by form-owner)
- `microservices/intelligence` UI overlays (CI runner reactions for review threads)
- `microservices/calendar` (event reactions — Yes/No/Maybe + custom)

### 1.5 Out-of-scope surfaces (v1)

- `microservices/meet` live overlay reactions (planned v1.1; reuse the same substrate, but with low-latency WebRTC datachannel transport; see `microservices/meet/PRD.md`)
- `microservices/recordings` post-call reaction timeline (v1.2)
- `microservices/observability` dashboards (no user-authored text)
- `microservices/api-gateway` (infrastructure surface)

### 1.6 Glossary

| Term | Definition |
|---|---|
| **Unicode emoji** | A code-point sequence emitted by the operating system or a font-substituted glyph (e.g., U+1F600). |
| **Sticker** | A packaged, named image (static or animated) belonging to a sticker pack; identified by `(pack_id, sticker_id)`. |
| **Sticker pack** | A versioned, signed, manifest-described set of stickers, distributable as a single ZIP archive. |
| **Custom emoji** | A tenant- or server-scoped image referenced by `:slug:`, distinct from Unicode emoji. |
| **Reaction** | An attachment of (Unicode emoji \| custom emoji \| sticker) to a message, comment, or post. |
| **Super-reaction** | An animated, time-bound, often-paid reaction with a particle / overlay effect (Discord parity). |
| **GIF** | A short looping animation; the on-disk format is *not* necessarily GIF (.gif) — see [§5](#5-gif-system). |
| **Tenor / Giphy** | Third-party hosted GIF libraries; oyatie integrates as a proxy. |
| **Pack registry** | Per-tenant relational catalog of installed packs. |
| **Pack blob store** | Per-cell SeaweedFS bucket holding pack binary blobs. |
| **Pack renderer** | Client-side rendering pipeline (no server bounded context). |
| **dotLottie** | A 2024 packaging standard (.lottie file) for Lottie animations + thumbnails + manifest. |

### 1.7 Authority chain

ESRG sits at the standards lane (`docs/standards/`) and is authoritative for any microservice that exposes user-authored text. When this document conflicts with a per-microservice PRD, the standard wins unless the microservice's PRD explicitly carves out a deviation with rationale and an ADR. The standard's owning team is the **Messenger** microservice owner by precedence (most-used surface), but the substrate is microservice-agnostic and could be extracted to a dedicated `microservices/expression` future µservice (see [§16](#16-bounded-contexts)) per ADR-0132 (per-microservice flat layout).

---

## 2. Unicode Emoji

### 2.1 Default render: Twemoji v15+ (CC-BY-4.0)

oyatie ships **Twemoji** as the default emoji glyph set across all surfaces, web and native, regardless of operating system. Rationale:

- **Cross-platform pixel parity**: a 👨‍👩‍👧‍👦 looks identical on a Windows laptop and an Android handset in the same DM thread. The user-perceived cost of cross-platform inconsistency (the iOS 🍑 vs. Android 🍑 problem) is non-trivial and Twemoji eliminates it.
- **License**: CC-BY-4.0 — permissive, attribution-only; satisfies enterprise compliance.
- **Active maintenance**: Twemoji has shipped Unicode 15.1 (2023) glyphs; community-maintained Unicode 16 (2024) follow-ons exist (`jdecked/twemoji` fork is the canonical maintained branch as of 2024).
- **Familiarity**: it is the dominant web-emoji style; Slack, Twitter (pre-X), Discord, GitHub, Mastodon, and Bluesky all use it.

### 2.2 Alternative: Noto Emoji (Google; SIL Open Font License 1.1)

Tenants may opt into **Noto Emoji** as the default. Rationale:

- Per-locale fallback completeness (Google maintains Noto Emoji with extreme breadth).
- SIL Open Font License — permissive.
- Sharper glyphs at small render sizes (12–16px) compared to Twemoji's softer style; useful for dense data surfaces (Sheets, Foundry).

### 2.3 Per-user override

A user may override the tenant default in their personal preferences:

- `system` — use OS-native emoji (Apple on macOS/iOS; Segoe UI Emoji on Windows; Noto on Android; system on Linux).
- `apple` — bundle Apple Color Emoji (license: per-platform; only legally usable on Apple platforms — enforced).
- `noto` — Noto Emoji.
- `twemoji` — Twemoji (default).
- `system-with-twemoji-fallback` — use OS-native where available; Twemoji for codepoints missing in the OS font (e.g., Unicode 16 on older Windows).

UI surface: `Settings → Appearance → Emoji style`.

Backend representation: stored on the user record as `emoji_font_preference: enum`. The tenant may forbid `apple` (e.g., for non-Apple-licensed environments).

### 2.4 Per-tenant override

A tenant admin may set the tenant-wide default and may *disable* user overrides (e.g., regulated tenants pin Twemoji for forensic consistency). The setting:

```yaml
tenant_settings:
  emoji_font:
    default: twemoji
    user_override_allowed: true
    forbidden_choices: [apple]   # apple cannot be selected on non-Apple platforms
```

### 2.5 Unicode 16 (2024) support

The substrate targets **Unicode 16.0** (released 2024-09) as its minimum supported emoji set on day one of v1, including:

- 7 new emoji glyphs added in 16.0 (face with bags under eyes, fingerprint, root vegetable, leafless tree, splatter, harp, shovel).
- Continued forward-compatibility track: when Unicode 17.0 (2025) ships, ESRG adds new glyphs within 90 days of public release.

The substrate maintains its emoji metadata catalog as `data/emoji/unicode-16.0.json`, regenerated by a build script that ingests:

- The Unicode CLDR (Common Locale Data Repository) emoji annotations at v46+ (50+ locales).
- The official Unicode emoji data files (`emoji-test.txt`, `emoji-data.txt`).

### 2.6 Skin-tone modifiers (Fitzpatrick 1-6)

Every emoji that supports skin-tone modifiers per Unicode UTS #51 must offer the picker for tones:

| Modifier | Codepoint | Fitzpatrick |
|---|---|---|
| (none) | — | Yellow (default) |
| 🏻 | U+1F3FB | I–II |
| 🏼 | U+1F3FC | III |
| 🏽 | U+1F3FD | IV |
| 🏾 | U+1F3FE | V |
| 🏿 | U+1F3FF | VI |

UX: long-press (mobile) / hover (desktop) on the base emoji exposes the tone selector. The user's most-recently-picked tone is persisted per emoji and recalled on next use (matches iOS/Android UX).

### 2.7 Gender-neutral options

Where Unicode provides gendered variants (e.g., 🧑‍🚀 / 👨‍🚀 / 👩‍🚀), the picker UI exposes:

- Gender-neutral (default).
- Male.
- Female.

The picker remembers per-user preference. The default is gender-neutral (matches Unicode's encouragement post-Unicode 12.0).

### 2.8 ZWJ sequences (emoji combinations)

The substrate renders Unicode Zero-Width-Joiner sequences as a single glyph:

- 👨‍👩‍👧‍👦 (family of four) — `U+1F468 U+200D U+1F469 U+200D U+1F467 U+200D U+1F466`
- 👩🏽‍💻 (woman technologist; medium skin) — base + modifier + ZWJ + role
- 🏳️‍🌈 (rainbow flag) — `U+1F3F3 U+FE0F U+200D U+1F308`

Rendering uses the OpenType `liga` / `clig` features for the font, or falls back to Twemoji's pre-rendered ZWJ sequence assets (Twemoji ships ZWJ sequences as discrete SVG files keyed by the codepoint sequence).

Search must understand ZWJ sequences: typing "family" should surface family compositions.

### 2.9 Component selection UI (composed emoji builder)

For complex emoji (especially professionals, families, couples), the substrate ships a **component builder** UI:

- Base emoji (e.g., 🧑 person).
- Skin tone (Fitzpatrick 1-6 or default).
- Gender (M / F / neutral).
- Role / modifier (technologist 💻, scientist 🔬, judge ⚖️, farmer 🌾, etc.).
- Hair (red 🦰, curly 🦱, white 🦳, bald 🦲) where applicable.

The builder previews the composed glyph and emits the ZWJ sequence. Recently-composed emoji are surfaced first.

### 2.10 Search

The picker exposes search by:

- **Name** (CLDR-localized): "smile", "thumbs up", "kiss".
- **Unicode codepoint**: `U+1F600`, `1F600`, `0x1F600`.
- **Category**: Smileys & Emotion, People & Body, Component, Animals & Nature, Food & Drink, Travel & Places, Activities, Objects, Symbols, Flags.
- **Subcategory**: face-smiling, face-affection, face-tongue, hand-fingers-open, etc. (per CLDR sub-groups).
- **Tag / keyword**: CLDR annotations (e.g., "joy" tagged on 😂).
- **Custom server emoji**: surfaces with prefix `:`-search (e.g., `:partyparrot:`).
- **Frequently used** (per user, last 30 days, top 50).
- **Recently used** (per user, last 16, in MRU order).

Search is locally-indexed (no server round-trip for emoji search) using a precomputed `data/emoji/search-index-<locale>.json` (gzip-compressed; ~400 KB per locale). The first-launch download is cached. Subsequent updates ship as delta patches.

---

## 3. Sticker System

### 3.1 Sticker pack format

A **sticker pack** is a versioned, signed archive containing static and/or animated stickers plus a manifest.

#### 3.1.1 On-disk layout (ZIP archive `.oyastk`)

```
my-pack.oyastk
├── pack.json                  # manifest (REQUIRED)
├── thumbnail.webp             # 96×96 static thumbnail (REQUIRED)
├── stickers/
│   ├── 001-happy.webp
│   ├── 002-sad.webp
│   ├── 003-dancing.webp       # animated WebP
│   ├── 004-rainbow.lottie     # dotLottie animated vector
│   ├── 005-fallback.apng      # APNG fallback for animated WebP
│   └── ...
├── previews/
│   ├── 001-happy-preview.webp # 128×128 preview
│   └── ...
├── i18n/
│   ├── en-US.json             # English strings
│   ├── ko-KR.json
│   ├── ja-JP.json
│   └── ...
├── LICENSE.txt                # human-readable license summary
└── signature.sig              # Ed25519 signature over pack.json (when distributed via marketplace)
```

#### 3.1.2 Manifest (`pack.json`) — JSON Schema (Draft 2020-12)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://oyatie.com/schemas/sticker-pack/v1.json",
  "title": "OyatieStickerPack",
  "type": "object",
  "required": [
    "schema_version",
    "pack_id",
    "name",
    "version",
    "author",
    "license",
    "stickers",
    "thumbnail"
  ],
  "properties": {
    "schema_version": {
      "type": "string",
      "const": "1.0",
      "description": "Pack manifest schema version (semver)."
    },
    "pack_id": {
      "type": "string",
      "pattern": "^[a-z0-9-]{3,64}$",
      "description": "Globally unique pack identifier. Slug-only."
    },
    "name": {
      "type": "object",
      "additionalProperties": { "type": "string" },
      "required": ["en-US"],
      "description": "Display name per locale; en-US REQUIRED."
    },
    "description": {
      "type": "object",
      "additionalProperties": { "type": "string" }
    },
    "version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+$",
      "description": "Pack version (semver)."
    },
    "author": {
      "type": "object",
      "required": ["name"],
      "properties": {
        "name": { "type": "string" },
        "url":  { "type": "string", "format": "uri" },
        "email": { "type": "string", "format": "email" },
        "creator_id": { "type": "string" }
      }
    },
    "license": {
      "type": "string",
      "description": "SPDX identifier or marketplace license slug.",
      "examples": ["CC-BY-4.0", "CC-BY-NC-4.0", "OYA-MARKETPLACE-EULA-v1"]
    },
    "min_app_version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+$"
    },
    "thumbnail": {
      "type": "string",
      "description": "Relative path to thumbnail (96x96 WebP)."
    },
    "category": {
      "type": "string",
      "enum": ["greetings", "emotions", "celebration", "love", "animals",
               "food", "work", "tech", "sports", "memes", "characters",
               "holidays", "regional-kr", "regional-jp", "regional-other",
               "custom", "other"]
    },
    "tags": {
      "type": "array",
      "items": { "type": "string", "maxLength": 32 },
      "maxItems": 32
    },
    "animated": { "type": "boolean" },
    "premium": { "type": "boolean", "default": false },
    "stickers": {
      "type": "array",
      "minItems": 1,
      "maxItems": 256,
      "items": {
        "type": "object",
        "required": ["id", "name", "file_path"],
        "properties": {
          "id": {
            "type": "string",
            "pattern": "^[a-z0-9-]{1,64}$"
          },
          "name": {
            "type": "object",
            "required": ["en-US"],
            "additionalProperties": { "type": "string" }
          },
          "file_path": {
            "type": "string",
            "description": "Relative path under stickers/."
          },
          "preview_path": {
            "type": "string"
          },
          "fallback_path": {
            "type": "string",
            "description": "APNG/PNG fallback for animated stickers."
          },
          "animated": { "type": "boolean", "default": false },
          "format": {
            "type": "string",
            "enum": ["webp", "apng", "png", "lottie", "dotlottie", "svg"]
          },
          "duration_ms": {
            "type": "integer",
            "minimum": 0,
            "maximum": 10000,
            "description": "Animation duration; 0 for static."
          },
          "loop": { "type": "boolean", "default": true },
          "width": { "type": "integer", "minimum": 1, "maximum": 1024 },
          "height": { "type": "integer", "minimum": 1, "maximum": 1024 },
          "byte_size": { "type": "integer", "minimum": 1 },
          "keywords": {
            "type": "array",
            "items": { "type": "string", "maxLength": 32 },
            "maxItems": 16
          },
          "emoji_associations": {
            "type": "array",
            "description": "Unicode emoji that map to this sticker for keyboard suggestion.",
            "items": {
              "type": "string",
              "description": "Unicode codepoint sequence (e.g., 1F600)."
            },
            "maxItems": 8
          },
          "alt_text": {
            "type": "object",
            "required": ["en-US"],
            "additionalProperties": { "type": "string" },
            "description": "Screen-reader text per locale; en-US REQUIRED."
          },
          "moderation": {
            "type": "object",
            "properties": {
              "rating": {
                "type": "string",
                "enum": ["G", "PG", "PG-13", "R", "X"]
              },
              "categories": {
                "type": "array",
                "items": { "type": "string" }
              }
            }
          }
        }
      }
    },
    "marketplace": {
      "type": "object",
      "properties": {
        "price_cents": { "type": "integer", "minimum": 0 },
        "currency": { "type": "string", "pattern": "^[A-Z]{3}$" },
        "subscription_required": { "type": "boolean", "default": false }
      }
    },
    "signature": {
      "type": "object",
      "properties": {
        "algorithm": { "type": "string", "const": "Ed25519" },
        "public_key": { "type": "string" },
        "signed_at": { "type": "string", "format": "date-time" },
        "value": { "type": "string" }
      }
    }
  }
}
```

#### 3.1.3 Pack sizes + limits

| Constraint | Free pack | Premium pack | Tenant-internal pack |
|---|---|---|---|
| Max stickers per pack | 32 | 256 | 256 |
| Max total pack size (post-compression) | 8 MB | 64 MB | 128 MB |
| Max per-sticker size (static WebP) | 256 KB | 512 KB | 1 MB |
| Max per-sticker size (animated WebP) | 1 MB | 4 MB | 8 MB |
| Max per-sticker size (Lottie/dotLottie) | 256 KB | 1 MB | 2 MB |
| Max sticker dimension | 512 × 512 | 1024 × 1024 | 1024 × 1024 |
| Max animation duration | 3 s | 6 s | 10 s |
| Loop max | 5 loops | infinite | infinite |

The pack-build pipeline (CLI tool `oya-stickers build`) validates all constraints and rejects packs violating them with a descriptive error.

### 3.2 Per-tenant sticker pack registry

Each tenant has a per-tenant registry of installed packs, persisted in Postgres (Citus-sharded per tenant; see [§7.2](#72-postgres-per-tenant-pack-registry)).

Cedar policy (per ADR-0243) gates every install:

```cedar
permit (
  principal,
  action == Action::"sticker.pack.install",
  resource
)
when {
  principal.tenant == resource.tenant &&
  principal.has_role("tenant_admin") || resource.pack_category in principal.allowed_pack_categories
};
```

The Cedar policy is authored per-tenant; the default policy permits any user to install any non-paid, non-NSFW pack. Stricter policies (e.g., regulated tenants) restrict installs to admin-approved packs.

### 3.3 Marketplace integration

Paid sticker packs are listed in the marketplace (`microservices/marketplace`), discoverable per ADR-0249. Pack purchase flow:

1. User browses marketplace, finds paid pack.
2. User clicks "Buy" — Cedar checks `marketplace.purchase.allowed` (tenant-admin may forbid users from purchasing direct-to-personal-payment-method packs).
3. User pays via the marketplace's payment integration (delegated to the payments substrate; post-payments-cert per ADR-0249's roadmap).
4. Marketplace mints an entitlement record: `(user_id, pack_id, expiry)`.
5. Pack registry observes entitlement event; installs pack for that user.
6. Pack blob (signed `.oyastk` archive) is downloaded to the user's installed cache (per-cell + edge CDN).

Subscription packs follow the same flow but with a recurring entitlement check.

### 3.4 Sticker pack categories (curated)

The substrate ships baseline packs across these categories (all free for v1):

| Pack | Stickers | Animated | Category |
|---|---:|---|---|
| Oyatie Originals | 64 | yes | greetings |
| Office Reactions | 32 | yes | work |
| Heart & Hugs | 24 | partly | love |
| Celebration Pack | 32 | yes | celebration |
| Tech Memes | 48 | yes | memes |
| KR Greetings | 48 | yes | regional-kr |
| JP Cute Pack | 48 | yes | regional-jp |
| EU Multi-Greet | 32 | partly | regional-other |
| Animal Friends | 32 | partly | animals |
| Food Reactions | 24 | partly | food |
| Sports Fan | 24 | yes | sports |
| Holiday Cheer | 32 | yes | holidays |

### 3.5 Sticker classes (parity grid)

| Class | Vendor parity | Spec hint |
|---|---|---|
| **Static stickers** | WhatsApp, Telegram | WebP static; 256 KB max |
| **Animated WebP stickers** | WhatsApp, Discord | Animated WebP; 1 MB max |
| **Lottie vector stickers** | Telegram (.tgs), LINE (apng-based) | dotLottie .lottie file |
| **APNG stickers** | Telegram fallback | APNG with WebP source |
| **Sound stickers** | Discord-Soundboard adjacent | Not v1; v2 roadmap |
| **3-D / AR stickers** | iMessage Memoji | Not v1 |
| **Custom server emojis** | Discord, Slack | See [§6](#6-custom-emoji--sticker-creation) |
| **Server-bound animated emoji** | Discord Nitro / Discord-only | Premium tier; see [§14](#14-marketplace--monetization) |

### 3.6 Sticker → emoji keyboard prediction

If a user types an emoji, the keyboard offers suggested stickers whose `emoji_associations` contain that emoji. This matches the iOS Memoji stickers / Tenor emoji → GIF behavior.

Performance budget: suggestion population ≤ 50 ms after emoji insert.

---

## 4. Reactions

### 4.1 Definition

A **reaction** is a (target, reactor, payload) triple:

- **Target** = a message, comment, post, or thread root (anywhere ESRG is enabled).
- **Reactor** = a user.
- **Payload** = a Unicode emoji codepoint sequence, OR a custom server emoji ref, OR a sticker ref `(pack_id, sticker_id)`, OR a super-reaction ref.

### 4.2 Standard quick-reaction bar

The default quick-reaction bar shows 7 reactions (industry standard):

`👍 ❤️ 🎉 🤔 😢 😄 🚀`

These are surfaced on hover (desktop) or long-press (mobile) over any reactable target. The user may customize the bar (Settings → Reactions → Quick bar) to any 7 emoji or custom emoji.

### 4.3 Reaction picker (full)

Beyond the quick-bar, a `+` button opens the full picker (same as the emoji/sticker picker; reuses [§2.10 search](#210-search)).

### 4.4 Super-reactions

Super-reactions are animated, particle-effect-bearing reactions that occupy more screen real estate momentarily. Examples:

- 🎉 confetti burst (covers ~60% of viewport for 1.2 s).
- 🔥 fire animation.
- ✨ sparkle trail.

Super-reactions are:

- **Tenant-policy-gated** (Cedar). Some tenants disable them as noise.
- **Per-server enabled/disabled**.
- **Rate-limited**: max 1 super-reaction per user per target per 60 s.
- **Premium-tier-gated** for some animation styles (`marketplace`).

### 4.5 Reaction count + reactor list

The UI shows reaction count and on hover/tap a list of reactors (latest 10 + "and N more"). Cedar gates reactor-list visibility:

- Public posts: visible to all viewers.
- Private DMs: visible only to thread participants.
- Channel posts: visible per channel ACL.
- Anonymous reactions (Community-anonymous-mode per `microservices/anonymous`): reactor list shows `Anonymous (n)`.

### 4.6 Per-message + per-thread reactions

Reactions attach to:

- A single message (default).
- A thread root (Slack-style; reaction reflects view of the whole thread).
- A comment under a doc/drive/sheet/slide (Docs-style inline).
- A post (Community).

Each surface's reaction storage is local (per its bounded context); ESRG provides the rendering + picker + payload-schema-validation libraries. Cross-microservice schema is canonicalized via `oya-shared-expression-protocol` (proto + AsyncAPI; see [§16](#16-bounded-contexts)).

### 4.7 Reaction shortcut

- **Mobile**: long-press 0.4 s on the target reveals the quick-bar in-place.
- **Desktop**: hover → reaction toolbar floats above the target.
- **Keyboard**: `+` + emoji name autocomplete (`+thumbs<TAB>` → 👍).

Slack-style shortcut also supported on desktop: `Ctrl/Cmd-Alt-R` opens reaction picker on the hovered target.

### 4.8 Customizable quick-reaction bar

Per-user setting:

```yaml
user_preferences:
  reactions:
    quick_bar:
      - "👍"
      - "❤️"
      - ":partyparrot:"      # custom server emoji
      - "🎉"
      - "🤔"
      - "😢"
      - ":server:mychannel:fire-anim"  # animated server emoji
```

Defaults to industry-standard 7. Custom server emojis allowed only on surfaces where that server's custom emojis are visible.

### 4.9 Reaction event schema

Inter-microservice event (canonical AsyncAPI; see `microservices/messenger/contracts/asyncapi/`):

```yaml
ReactionAdded:
  type: object
  required: [event_id, tenant_id, target, actor, payload, occurred_at]
  properties:
    event_id: { type: string, format: uuid }
    tenant_id: { type: string }
    target:
      type: object
      required: [surface, target_id]
      properties:
        surface: { type: string, enum: [messenger, community, docs, drive, mail, slides, sheets, tasks, social, notes, forms, calendar] }
        target_id: { type: string }
        thread_id: { type: string }
    actor:
      type: object
      required: [actor_id]
      properties:
        actor_id: { type: string }
        anonymous: { type: boolean, default: false }
    payload:
      oneOf:
        - $ref: "#/components/schemas/UnicodeEmojiPayload"
        - $ref: "#/components/schemas/CustomEmojiPayload"
        - $ref: "#/components/schemas/StickerPayload"
        - $ref: "#/components/schemas/SuperReactionPayload"
    occurred_at: { type: string, format: date-time }
```

```yaml
UnicodeEmojiPayload:
  type: object
  required: [type, codepoints]
  properties:
    type: { type: string, const: unicode }
    codepoints: { type: string, description: "Hex-encoded codepoint sequence, e.g., 1F600 or 1F468-200D-1F469-200D-1F467" }
    skin_tone: { type: string, enum: [none, 1-2, 3, 4, 5, 6] }

CustomEmojiPayload:
  type: object
  required: [type, ref]
  properties:
    type: { type: string, const: custom_emoji }
    ref: { type: string, description: "tenant:slug or tenant:server:slug" }
    version: { type: string }

StickerPayload:
  type: object
  required: [type, pack_id, sticker_id]
  properties:
    type: { type: string, const: sticker }
    pack_id: { type: string }
    sticker_id: { type: string }
    pack_version: { type: string }

SuperReactionPayload:
  type: object
  required: [type, super_id]
  properties:
    type: { type: string, const: super_reaction }
    super_id: { type: string }
    duration_ms: { type: integer, minimum: 100, maximum: 5000 }
```

### 4.10 Reaction limits

| Limit | Value |
|---|---|
| Max reactions per user per target | 16 distinct emoji |
| Max distinct reactions per target | 50 (after which "load more" toggle) |
| Rate limit per user (any surface) | 600 reactions/hour |
| Rate limit super-reactions per user | 60/hour |

---

## 5. GIF System

### 5.1 GIF search via Tenor (primary) + Giphy (fallback)

The substrate integrates two third-party GIF providers as primary + fallback:

- **Tenor** (Google-owned; free API tier; per Tenor API docs).
  - API key per tenant (or shared key for default tenant tier).
  - Search endpoint: `https://tenor.googleapis.com/v2/search`.
  - Trending: `https://tenor.googleapis.com/v2/featured`.
  - Categories: `https://tenor.googleapis.com/v2/categories`.
  - Per-locale results.
- **Giphy** (Meta-adjacent; free API tier with attribution).
  - Used as fallback when Tenor returns empty results or Tenor API is unavailable.

### 5.2 Per-tenant GIF source allowlist

A tenant may configure which sources are allowed:

```yaml
tenant_settings:
  gif_sources:
    enabled: [tenor]              # disable giphy globally
    self_hosted_only: false       # set true for air-gapped tenants
    allowed_categories: [reactions, celebration, animals, sports]
    blocked_queries: ["nsfw", "violence"]      # blocked search terms
    rating_max: "PG-13"           # filter results by Tenor/Giphy rating
```

### 5.3 Self-hosted GIF library (air-gapped tenants)

Regulated tenants (e.g., government, healthcare-PHI-restricted) may run with **no external GIF sources**. ESRG ships:

- A `microservices/messenger/iac/gif-mirror/` per-cell Tenor-mirror seeder (offline pre-loaded curated library of ~10,000 GIFs).
- Stored in SeaweedFS per cell; CDN-fronted.
- Tenant admin can upload custom GIFs to the tenant-private library.

### 5.4 GIF rendering format strategy

The GIF format on-disk (`.gif`) is *not* what we render. We render the best-compression format the client supports:

| Client | First choice | Second choice | Third choice |
|---|---|---|---|
| Web (modern) | animated WebP | animated AVIF | actual GIF |
| Web (legacy) | animated WebP | APNG | actual GIF |
| iOS / macOS native | animated WebP via SDWebImage | APNG | actual GIF |
| Android native | animated WebP via Coil | APNG | actual GIF |
| Windows native | animated WebP via WebView2 | APNG | actual GIF |
| Linux GTK / Qt | animated WebP via libwebp | APNG | actual GIF |

Tenor and Giphy both expose multiple format URLs per result; the ESRG proxy selects the best match and caches per-cell.

### 5.5 GIF proxy bounded context

ESRG runs a `gif-proxy` bounded context (logical) on the messenger microservice (extractable to dedicated µservice per [§16](#16-bounded-contexts)) that:

- Accepts client search request.
- Forwards to Tenor (primary) or Giphy (fallback).
- Applies tenant policy (allowed_categories, rating filter, blocked_queries).
- Caches results per (tenant, locale, query) for 24 h.
- Streams GIF binaries through the proxy, never letting the client touch Tenor/Giphy directly (preserves user IP privacy; rate-limits per tenant).
- Caches binaries per-cell on SeaweedFS with TTL 30 days.

Cedar policy: `permit ... action == Action::"gif.search"` is the default.

### 5.6 Per-tenant content moderation (per ADR-0251)

The trust-safety substrate (referenced in ADR-0251 compliance pack levels) classifies fetched GIFs and stickers. Default classifications:

- `G`: General audience.
- `PG`: Mild content; default acceptable.
- `PG-13`: Some mature themes; opt-in per tenant.
- `R`: Mature; gated to adult-tenant-confirmed users only.
- `X`: Explicit; default-blocked; only allowed on tenant-explicit-opt-in.

The classifier uses Tenor's `contentfilter` parameter (off, low, medium, high) and Giphy's `rating` parameter (g, pg, pg-13, r). For self-hosted GIFs, the classifier is an Intelligence Substrate vision model (opt-in per tenant per ADR-0251).

---

## 6. Custom Emoji + Sticker Creation

### 6.1 Scope

Three scopes for custom emoji + sticker:

1. **Per-server / per-community / per-channel** (Discord-style) — uploaded by community/channel admins; visible only within that scope.
2. **Per-tenant** — uploaded by tenant admin; visible across the tenant, across all surfaces.
3. **Per-user (personal)** — uploaded by the user; visible only to the uploader (rare; v1 supports for personal stickers in DM only).

### 6.2 Upload formats

| Format | Allowed | Notes |
|---|---|---|
| PNG (static) | yes | Max 1024×1024; auto-resized to 128×128 baseline. |
| WebP (static) | yes | Preferred; smallest. |
| WebP (animated) | yes | Auto-validates ≤ 1 MB; ≤ 10 s; ≤ 60 fps. |
| APNG | yes | Converted to animated WebP internally. |
| Lottie (.json) | yes | Validated against Lottie 5.5+ spec. |
| dotLottie (.lottie) | yes | 2024 spec; preferred for vector animation. |
| GIF | yes | Converted to animated WebP; rejected if > 2 MB pre-conversion. |
| SVG (static) | yes (per-tenant only) | Rejected if contains script tags; sanitized. |

### 6.3 Size limits

| Constraint | Limit |
|---|---|
| Static custom emoji | 256 KB |
| Animated custom emoji | 1 MB |
| Custom sticker (static) | 512 KB |
| Custom sticker (animated) | 4 MB |
| Per-server emoji count (free) | 250 |
| Per-server emoji count (premium) | 500 |
| Per-server emoji count (expanded, paid) | 1000 |
| Per-tenant emoji count | 10,000 (configurable) |
| Per-user personal stickers | 100 |

### 6.4 Naming convention

`:slug:` form (Slack/Discord convention). Constraints:

- Lowercase ASCII alphanumeric + underscore + dash.
- 2-32 characters.
- Must be unique within its scope.
- Reserved prefixes: `oya_`, `system_`, `unicode_` cannot be used.

Regex: `^[a-z0-9][a-z0-9_-]{1,30}[a-z0-9]$`.

### 6.5 Approval workflow (Cedar-gated)

```cedar
permit (
  principal,
  action == Action::"custom_emoji.upload",
  resource
)
when {
  principal.tenant == resource.tenant &&
  (
    principal.has_role("server_admin", resource.scope_id) ||
    principal.has_role("tenant_emoji_curator")
  )
};
```

The upload UX:

1. User uploads file via picker.
2. Frontend validates (size, format).
3. Backend re-validates (size, format, content scan, virus scan).
4. Trust-safety classifier scores (NSFW, hate, violence).
5. If score above threshold: admin approval queue.
6. If approved: published; visible in server/tenant emoji picker.
7. Audit event emitted to compliance substrate.

### 6.6 Per-server emoji limit + expansion

Default tier (free): 250 emojis per server.
Premium tier: 500.
Expanded tier (paid per-server upgrade): 1000.

A server admin sees usage in `Server Settings → Emojis → Usage` (e.g., "187 / 250 used").

### 6.7 Custom emoji animated variants

A custom emoji may have:

- **Static** version (default rendering).
- **Animated** version (rendered in reactions, in compose, in messages).

When the user inserts via `:slug:`, ESRG resolves to the highest-fidelity version supported by the receiving client. (Some clients with reduced-motion preference render the static version.)

---

## 7. Storage

### 7.1 Sticker pack blobs (SeaweedFS per cell)

Per-cell SeaweedFS bucket: `s3://oya-stickers-{cell-id}/packs/{tenant_id}/{pack_id}/{version}/`.

Bucket layout:

```
oya-stickers-cell-kr1/
  packs/
    {tenant_id}/
      {pack_id}/
        {version}/
          pack.oyastk          # signed archive
          manifest.json        # extracted manifest for indexing
          stickers/            # extracted blobs (CDN-served)
            *.webp
          previews/
            *.webp
          thumbnail.webp
```

Replication factor: 3 within cell. Cross-cell replication for packs marked `global=true`.

### 7.2 Postgres: per-tenant pack registry

Citus-sharded `oya-stickers-registry` Postgres database:

```sql
-- distribution key: tenant_id

CREATE TABLE pack_registry (
  tenant_id          UUID NOT NULL,
  pack_id            TEXT NOT NULL,
  version            TEXT NOT NULL,
  installed_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  installed_by       UUID NOT NULL,
  status             TEXT NOT NULL CHECK (status IN ('active', 'disabled', 'pending', 'rejected')),
  entitlement_id     UUID,
  metadata           JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, pack_id)
);
SELECT create_distributed_table('pack_registry', 'tenant_id');

CREATE TABLE user_pack_subscriptions (
  tenant_id    UUID NOT NULL,
  user_id      UUID NOT NULL,
  pack_id      TEXT NOT NULL,
  subscribed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at   TIMESTAMPTZ,
  source       TEXT NOT NULL,  -- marketplace | tenant_admin | gift
  PRIMARY KEY (tenant_id, user_id, pack_id)
);
SELECT create_distributed_table('user_pack_subscriptions', 'tenant_id');

CREATE TABLE custom_emoji (
  tenant_id      UUID NOT NULL,
  scope_id       UUID NOT NULL,  -- server_id or NULL for tenant-scope
  slug           TEXT NOT NULL,
  version        INT NOT NULL DEFAULT 1,
  blob_path      TEXT NOT NULL,
  animated_path  TEXT,
  uploader_id    UUID NOT NULL,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  approval_status TEXT NOT NULL CHECK (approval_status IN ('pending', 'approved', 'rejected')),
  PRIMARY KEY (tenant_id, scope_id, slug)
);
SELECT create_distributed_table('custom_emoji', 'tenant_id');

CREATE TABLE reactions (
  tenant_id      UUID NOT NULL,
  target_id      TEXT NOT NULL,
  surface        TEXT NOT NULL,
  actor_id       UUID NOT NULL,
  payload_kind   TEXT NOT NULL,
  payload_value  JSONB NOT NULL,
  reacted_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, target_id, actor_id, payload_kind, payload_value)
);
SELECT create_distributed_table('reactions', 'tenant_id');

CREATE INDEX reactions_target_idx ON reactions (tenant_id, target_id);
CREATE INDEX reactions_actor_idx ON reactions (tenant_id, actor_id);
```

### 7.3 CDN delivery (Cloudflare R2 + edge cache)

Sticker blobs and custom emoji are served via Cloudflare R2 (per-cell origin) with:

- Edge cache TTL: 30 days for pack blobs; 7 days for custom emoji (lower because of admin edits).
- Cache-Control: `public, max-age=2592000, immutable` (immutable on content-hash URLs).
- Signed URL for premium packs (entitlement-gated).
- Range request support (for animated WebP scrubbing).

### 7.4 Fingerprint-based dedup

Every uploaded blob is content-addressed by SHA-256. The pack-build pipeline:

1. Computes SHA-256.
2. Checks against the per-tenant blob registry.
3. If exists: reuses; pack manifest points at existing blob.
4. If new: uploads.

This deduplicates re-uploads of identical artwork (saves 10-30% storage on average across the marketplace).

### 7.5 Encryption at rest for personal sticker packs

Per-user personal sticker packs (DM-only; see [§6.1](#61-scope)) are encrypted at rest using per-user keys derived from the user's MLS device key (per the Messenger MLS E2EE doc; see `microservices/messenger/policy/`).

Pack metadata remains server-readable (for listing the user's packs); pack *blobs* are E2EE.

### 7.6 Garbage collection

Packs uninstalled by every user are GC'd after 30 days of zero references. Packs deleted by a creator are tombstoned for 90 days then GC'd.

Custom emoji deleted by a server admin are tombstoned for 30 days then GC'd. Past message references continue to render until GC; after GC they show the `[:deleted-emoji:]` fallback.

### 7.7 Data sovereignty

For tenants in jurisdictions requiring data residency (KR, EU per GDPR Article 44+ transfers), the per-tenant pack registry + blobs are pinned to the jurisdiction's cell. No cross-jurisdiction replication.

---

## 8. Cross-platform Rendering

### 8.1 Web (browsers + Electron)

Stack:

- **React 18+** with the `oya-expression-web` SDK.
- **Twemoji v15.1+** glyph delivery via the `@twemoji/parser` library; SVG-by-default for crisp scaling; PNG fallback for legacy browsers.
- **Animated WebP** via the native `<picture>` + `<img>` with WebP source set (all modern browsers support animated WebP since Safari 16 in 2022).
- **APNG fallback** for IE/legacy (we don't support IE, but APNG covers old Firefox forks).
- **Lottie animations** via `@lottiefiles/dotlottie-web` (the 2024 dotLottie player; ~80 KB gzipped).
- **SVG** for vector custom emoji (sanitized server-side).

CSS budget: emoji glyphs render in a 1em line box; sticker preview at 128×128; sticker insert at 200×200 (high-DPI 400×400 source); GIF preview at 240×180 (low resolution); GIF insert at 360×270.

### 8.2 iOS / iPadOS (native)

Stack:

- **SwiftUI** + UIKit; the `OyaExpression` Swift package.
- **Twemoji or system emoji** per user/tenant preference; rendered via `NSAttributedString` with custom font fallback (Twemoji TTF embedded; ~6 MB).
- **Animated WebP** via **SDWebImage** + `SDWebImageWebPCoder`.
- **Lottie** via the **Lottie-iOS** library (Airbnb; widely-used; LICENSE Apache-2.0) — supports dotLottie 2024 via `lottie-ios` v4.4+.
- **Sticker drawer** matches Messages.app: scroll-pageable horizontal pack carousel.
- **Reactions** appear as floating bubble overlays; long-press to invoke (matches iMessage UX).

### 8.3 Android

Stack:

- **Jetpack Compose** + Kotlin; the `OyaExpression` Android library.
- **Twemoji or system Noto Emoji** per preference.
- **Animated WebP** via **Coil** (modern Android image library) — Coil 2.x supports animated WebP since 2022.
- **Glide** as alternate (older codebases).
- **Lottie** via the **Lottie-Android** library (Airbnb) — dotLottie support via lottie-android 6.0+.
- **Sticker drawer** matches WhatsApp Android: vertical scroll with sticky pack header.

### 8.4 macOS (native)

Stack:

- **AppKit** + SwiftUI; the `OyaExpression-macOS` Swift package (mostly shares iOS code).
- **Twemoji** or **system emoji**.
- **Animated WebP** via SDWebImage-macOS.
- **Lottie** via Lottie-macOS.

### 8.5 Windows (native + Electron)

Stack:

- **WinUI 3** for full-native (long-term roadmap; v1.1).
- **WebView2** for embedded web rendering (v1.0); shares the web stack.
- **Twemoji or Segoe UI Emoji** per user preference.
- **Animated WebP** via WebView2 native rendering.
- **Lottie** via dotlottie-wasm or via Skia.

### 8.6 Linux (native)

Stack:

- **GTK4** for GNOME-native (v1.1).
- **Qt6** for KDE-native (v1.1).
- **Electron** for v1.0 (shares web stack).
- **Twemoji** as default font.
- **Animated WebP** via libwebp.
- **Lottie** via lottie-rust + Skia.

### 8.7 CLI / TUI / terminal surfaces

Terminal surfaces (oya CLI; Foundry runner TUI):

- Unicode emoji rendered via the terminal's emoji font (system).
- Stickers rendered as **placeholder labels** (`[sticker: party_parrot]`).
- Reactions rendered as a numeric tally: `👍 5 ❤️ 3 :partyparrot: 1`.

### 8.8 Email rendering (Mail microservice + outbound)

Outbound email rendering (e.g., notification emails reflecting a reaction):

- Unicode emoji rendered as Unicode (recipient's email client decides).
- Stickers rendered as **inline image attachments** with `<img alt="...">` + fallback alt text.
- Animated stickers degrade to **static first-frame** in outbound email (animation-in-email is unreliable cross-client).
- GIFs rendered as actual `.gif` for max email-client compatibility.

### 8.9 Print + PDF export (Docs, Slides)

When exporting a Doc or Slide to PDF:

- Unicode emoji rendered via the embedded Twemoji SVG (vector).
- Stickers rendered as their static first frame (or static thumbnail).
- Reactions rendered as a numeric tally string.
- Custom emoji rendered with alt text + image embed.

---

## 9. Performance

### 9.1 Performance budgets (p50 / p95 / p99)

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| Sticker render (single frame) | ≤ 8 ms | ≤ 16 ms | ≤ 32 ms |
| Sticker animation (frame budget) | ≤ 16 ms/frame (60 fps) | ≤ 33 ms/frame (30 fps) | ≤ 50 ms/frame (20 fps) |
| Sticker picker open | ≤ 50 ms | ≤ 100 ms | ≤ 200 ms |
| Emoji picker open | ≤ 30 ms | ≤ 80 ms | ≤ 150 ms |
| GIF search (client → render first result) | ≤ 200 ms | ≤ 300 ms | ≤ 600 ms |
| Reaction add (optimistic) | ≤ 16 ms | ≤ 32 ms | ≤ 50 ms |
| Reaction server-sync | ≤ 100 ms | ≤ 200 ms | ≤ 400 ms |
| Pack download (progressive: static visible) | ≤ 200 ms | ≤ 500 ms | ≤ 1.5 s |
| Pack download (full: all animations loaded) | ≤ 2 s | ≤ 5 s | ≤ 15 s |
| Custom emoji upload + approval-queue write | ≤ 800 ms | ≤ 2 s | ≤ 5 s |
| Lottie animation (first frame) | ≤ 30 ms | ≤ 60 ms | ≤ 120 ms |

### 9.2 Optimistic reaction add

The client renders the reaction *immediately* upon user tap (optimistic). The reaction is sent to the server; if the server rejects (Cedar policy denies; rate-limited; offline), the client rolls back with a brief shake animation.

### 9.3 Progressive pack download

On pack install:

1. **Phase 1** (≤ 200 ms): manifest + thumbnail + first 8 static stickers fetched; picker becomes usable.
2. **Phase 2** (background): remaining static stickers + previews.
3. **Phase 3** (background, low-priority): animated stickers, Lottie, large blobs.

The user can use any sticker that has loaded; un-loaded stickers show a shimmer placeholder.

### 9.4 Animation efficiency

Animated WebP is the format of choice because:

- Better compression than GIF (avg 50-70% smaller for equal quality).
- Hardware-accelerated decode on most platforms.
- Alpha channel support (GIF only has binary alpha).

Lottie (vector) is even smaller (~10-50 KB per sticker vs. 500 KB for raster animation) — used for sticker creators who design in After Effects / Rive.

### 9.5 Sticker LRU cache

The client maintains an LRU cache of decoded sticker frames:

- Web: IndexedDB-backed; 100 MB cap.
- iOS / macOS: file-system cache via SDWebImage; 200 MB cap.
- Android: Coil disk cache; 200 MB cap.

On low-memory eviction signal, the cache shrinks to 50% of its cap.

### 9.6 GIF caching

GIF binaries cached:

- Client-side: 50 MB cap; LRU.
- Server proxy: 30 days; per-cell SeaweedFS.

### 9.7 Reduced animation under load

When the client detects:

- Battery < 20%.
- Low-memory pressure.
- User preference `reduce-motion` (OS-level).
- Network bandwidth saturated.

…it switches to static-only mode (animated stickers render their first frame; animated reactions render static).

### 9.8 Server-side picker recommendation

The "recently used" + "frequently used" emoji and stickers are computed client-side; for shared accounts and multi-device users, the server keeps a per-user MRU list synced via the user-prefs substrate.

---

## 10. Accessibility

### 10.1 Alt-text mandatory

Every sticker, custom emoji, and GIF must carry alt-text. The pack manifest's `alt_text` field is **required** for marketplace publication. Custom emojis: uploader must provide alt-text at upload time.

Default fallback: if no alt-text, the rendered `<img>` carries `alt=":slug:"` for custom emojis, and `alt="sticker"` for sticker library defaults — but the marketplace approval pipeline rejects packs without proper alt-text.

### 10.2 Screen-reader-friendly emoji names

Unicode emoji are announced by their CLDR-localized name (e.g., 😀 → "grinning face"). The locale is the user's preference.

### 10.3 Reduced-motion

Respects:

- Web: `prefers-reduced-motion: reduce` CSS media query.
- iOS: `UIAccessibility.isReduceMotionEnabled`.
- Android: `Settings.Global.ANIMATOR_DURATION_SCALE = 0` or `Settings.Global.TRANSITION_ANIMATION_SCALE = 0`.
- macOS: `NSWorkspace.shared.accessibilityDisplayShouldReduceMotion`.

When reduced-motion is set:

- Animated stickers render as static (first frame).
- Super-reactions render as static (no particle effects).
- Custom animated emojis render as static.
- Animated GIFs play but with a "click to play/pause" overlay; default state is paused.

The user may opt in to animations even with system-reduced-motion via a per-app override (Settings → Accessibility → Animations).

### 10.4 Color-blind support

Emoji metadata includes color descriptions for the 8 most-common emojis where color matters (e.g., 🔴 red circle vs. 🟢 green circle for status). The picker tooltip surfaces the description.

### 10.5 Search by name (always available)

The picker must always allow text search by name. Users with vision impairment may not be able to scan a 1000-glyph grid; the keyboard-driven search ensures equal access.

### 10.6 Keyboard navigation

- Tab cycles through picker categories.
- Arrow keys move within the current category grid.
- Enter selects.
- Esc closes picker.
- Type-to-search.

### 10.7 Contrast + size

- Picker glyphs render at minimum 32×32 logical pixels (matches WCAG 2.2 target size).
- High-contrast theme available (picker background fully opaque; no transparency).
- Glyph borders visible at 1px in high-contrast mode (for color-blind discrimination).

### 10.8 Focus indicators

Each picker cell shows a 3px outline on focus (WCAG 2.4.7 conformant).

### 10.9 Voice control

iOS / macOS voice control: the user can say "tap the smile emoji" and the picker selects 😀 (matches the alt-text name).

---

## 11. Localization

### 11.1 Per-locale sticker pack recommendations

Tenant defaults adapt by user locale:

| Locale | First-shown pack |
|---|---|
| `ko-KR` | KR Greetings pack + KakaoTalk-style cute pack |
| `ja-JP` | JP Cute Pack + LINE-import pack (where available) |
| `en-US` | Oyatie Originals + Tech Memes |
| `en-GB` | Oyatie Originals + British Wit pack |
| `de-DE` | Oyatie Originals + EU Multi-Greet |
| `fr-FR` | Oyatie Originals + EU Multi-Greet |
| `zh-CN` | Oyatie Originals + CN-suitable pack (compliance-aware) |
| `pt-BR` | Oyatie Originals + LatAm pack |
| `es-ES` | Oyatie Originals + ES pack |
| `vi-VN` | Oyatie Originals + VN pack |
| (other) | Oyatie Originals only |

The locale-pack mapping ships with the substrate as `data/locale-pack-defaults.json`.

### 11.2 Emoji name translations (CLDR)

Emoji names + keywords from Unicode CLDR v46+ (~50 locales). Loaded per-user-locale; ~80 KB per locale gzipped.

### 11.3 Sticker pack i18n strings via Fluent + ICU

Pack manifest's `name`, `description`, `category`, sticker `name`, sticker `alt_text` are all locale-keyed objects. Format follows Project Fluent ([Mozilla Fluent](https://projectfluent.org/)) syntax with ICU MessageFormat fallback.

Example sticker pack i18n:

```json
{
  "name": {
    "en-US": "Office Reactions",
    "ko-KR": "오피스 리액션",
    "ja-JP": "オフィスリアクション",
    "zh-CN": "办公室反应",
    "de-DE": "Büro-Reaktionen"
  },
  "stickers": [
    {
      "id": "001-thumbs-up",
      "name": {
        "en-US": "Thumbs Up",
        "ko-KR": "엄지척",
        "ja-JP": "サムズアップ"
      },
      "alt_text": {
        "en-US": "Animated thumbs up gesture",
        "ko-KR": "움직이는 엄지손가락 위로 제스처",
        "ja-JP": "アニメーションのサムズアップジェスチャー"
      }
    }
  ]
}
```

### 11.4 RTL support

Stickers don't need RTL flipping per the W3C i18n best practices (icons retain orientation; flags retain origin); however, the picker UI layout flips for RTL locales (Arabic, Hebrew, Persian, Urdu).

### 11.5 Date / time formatting in sticker overlays

Stickers with overlaid text (e.g., a "Happy Birthday" pack) render per the user's locale's text direction and font conventions.

### 11.6 Locale-specific availability

Per-jurisdiction sticker pack availability:

- KR: includes packs with hangul; excludes packs banned by KCC.
- JP: includes katakana / hiragana / kanji packs.
- DE / EU: excludes packs with imagery banned by §86a StGB (Nazi imagery).
- CN: severely restricted; only oyatie-baseline packs.

The availability matrix lives in `microservices/marketplace/specs/jurisdiction-availability.json`.

---

## 12. Moderation

### 12.1 Per-tenant Cedar gate

Cedar policies (per ADR-0243) gate every sticker, GIF, and custom-emoji operation:

```cedar
permit (
  principal,
  action == Action::"sticker.pack.install" |
            Action::"custom_emoji.upload"  |
            Action::"gif.send"             |
            Action::"reaction.add",
  resource
)
when {
  principal.tenant == resource.tenant &&
  resource.content_rating <= principal.tenant.max_content_rating &&
  !resource.is_blocked_for(principal.tenant)
};
```

### 12.2 Trust-safety substrate

Per ADR-0249 / 0251 (compliance pack levels), stickers + GIFs + custom emoji are classified by the **Intelligence Substrate** trust-safety pipeline:

- **NSFW**: nudity, sexual content.
- **Violence**: graphic violence, weapons in aggressive context.
- **Hate**: hate symbols, slurs.
- **Self-harm**: imagery related to self-harm.
- **CSAM**: CSAM-classified content is auto-removed + reported per legal jurisdiction.
- **Spam**: bulk-uploaded promotional content.
- **Copyright**: known-copyrighted-imagery via fingerprint match (e.g., Disney characters).

Classification runs:

- **Pre-publication** on marketplace packs.
- **Pre-upload** on custom emoji.
- **Streaming** on GIF search (Tenor / Giphy results are pre-classified by their providers; oyatie re-checks high-rated content).

### 12.3 Per-tenant allow/deny lists

Tenant admins maintain:

- **Allow-list**: packs that bypass classification (e.g., enterprise-branded packs).
- **Deny-list**: packs/categories/queries always blocked.

### 12.4 Auto-moderation (opt-in)

Per-server / per-channel auto-moderation:

- **LLM-based content scoring** (via Intelligence Substrate).
- **Image-classifier scoring** (vision model).
- **Rule-based** (e.g., "any reaction with 🔫 is removed from #youth-channels").

Auto-moderation is **opt-in** per channel/server (per ADR-0251).

### 12.5 Report flow

Any user may report a sticker, GIF, custom emoji, or reaction:

1. Long-press / right-click → Report.
2. Reason picker (NSFW, hate, harassment, spam, copyright, other).
3. Optional comment.
4. Submitted to trust-safety queue.
5. Reviewed by trust-safety team (or automated for high-confidence violations).
6. Outcome: removed, warned, no-action.
7. Reporter notified of outcome (optional; not for low-confidence dispositions to prevent abuse).

### 12.6 Server-level emoji moderation

Server admins can:

- Approve / reject pending custom emoji.
- Disable specific emoji.
- Delete emoji (tombstone for 30 days; full GC after).
- Restrict who can react (e.g., only members ≥ 7 days old).

### 12.7 Bulk takedown

Trust-safety team has tooling to:

- Remove a pack across all tenants (e.g., copyright claim).
- Remove a specific sticker from a pack (and ripple-delete from messages where it appears, or replace with placeholder).
- Remove a custom emoji from a tenant.

All bulk operations emit audit events to the compliance substrate.

### 12.8 Appeals

Creators whose packs are removed may appeal:

- Submit appeal via marketplace creator portal.
- Reviewed within 5 business days.
- Outcome: reinstated, modified, denied.

---

## 13. Compliance

### 13.1 Per-jurisdiction availability

| Jurisdiction | Notable restrictions |
|---|---|
| KR | KCC content classifications; some explicit content tiers blocked |
| JP | Public decency laws; very-explicit content blocked |
| DE / EU | §86a StGB (no Nazi imagery); GDPR data residency |
| US | DMCA copyright + CSAM laws |
| CN | Severe restrictions; only oyatie-baseline + government-approved packs |
| SA / AE | Religious-content scrutiny; some imagery blocked |
| RU | (Operationally complex; outside v1 scope) |
| IN | Per IT Rules 2021; some content restrictions |

The marketplace availability matrix is the source of truth; per ADR-0249.

### 13.2 Per-tenant content policy

A tenant defines:

- Allowed pack categories.
- Forbidden pack categories.
- Custom emoji approval workflow.
- Trust-safety classifier sensitivity.
- Reaction restrictions (e.g., disable super-reactions).

### 13.3 Copyright

- Pack creators warrant they own all IP in the pack (per ADR-0249 marketplace creator agreement).
- DMCA / equivalent takedown flow available to rights-holders.
- Fingerprint-matching against known-copyrighted imagery (e.g., Disney; major sports leagues; major brands).
- Repeat infringers → creator account banned.

### 13.4 GDPR Article 17 (right to erasure)

A user requesting erasure:

- All custom emoji uploaded by the user are deleted (tombstoned 30 days; GC'd).
- All personal sticker packs are deleted.
- All reactions authored by the user are anonymized (sender_id replaced with sentinel) or deleted per the policy of the surface (Messenger E2EE messages are unrecoverable).
- Past messages where the user reacted: reaction count decremented; the reactor-list no longer includes the user.

### 13.5 Data residency

Per ADR-0251 cell-certification levels, KR-tenant sticker blobs stay in KR cells. EU-tenant stays in EU cells. US-tenant stays in US cells. Cross-region replication only for global packs and only after explicit tenant opt-in.

### 13.6 Audit

Every sticker install, emoji upload, reaction add, GIF send is audit-logged to the compliance substrate (per ADR-0251 cell-certification audit chain). The chain is cryptographically anchored (per messenger's audit-chain bounded context).

### 13.7 PII in custom emoji

Custom emoji + stickers are user-content; the upload pipeline:

- Strips EXIF (including geo-location, camera serial).
- Re-encodes (forces re-encode to avoid stego).
- Flags potential PII (face detection alerts admin; opt-in).

### 13.8 Children + COPPA / KOSA

Tenants serving users under 13 (per COPPA / KOSA / KR equivalents):

- Marketplace purchases require guardian consent.
- Custom emoji upload restricted.
- DM / direct sticker exchange restricted.
- Default content rating = G.

### 13.9 HIPAA-tagged tenants

Healthcare-tenant compliance:

- No emoji / sticker / GIF content uploaded to PHI-tagged conversations except oyatie-baseline.
- Trust-safety + classifier scans HIPAA-elevated.
- Audit logging extra-strict.

---

## 14. Marketplace + Monetization

### 14.1 Pack tiers

| Tier | Price | Limit | Examples |
|---|---|---|---|
| Free baseline | $0 | unlimited | Oyatie Originals; Office Reactions; KR Greetings |
| Free creator-published | $0 | unlimited | Indie-creator free packs |
| Paid one-time | $0.99–$9.99 | per-purchase | LINE-style premium packs |
| Subscription (Premium) | bundled in Premium | unlimited | Premium subscribers access all |
| Tenant-internal | $0 (internal) | unlimited within tenant | Corporate-branded packs |

### 14.2 Creator revenue share

Per ADR-0249 marketplace economics:

- **Free packs**: no revenue; creator paid via marketplace placement / featured listing optional reward.
- **Paid packs**: 70% creator / 30% oyatie (matches App Store standard).
- **Subscription packs**: pro-rata distribution based on usage analytics (number of unique downloads × time used).
- **Promotional packs**: creator receives flat-fee per oyatie marketing campaign.

### 14.3 Subscription model

Premium subscription (per `microservices/marketplace`) grants:

- All Premium packs.
- 500 custom emojis per server (up from 250).
- Animated server emojis.
- Super-reactions (all styles).
- Tenor + Giphy ad-free.
- Custom emoji size +50%.

### 14.4 Tenant-internal packs

A tenant may publish packs only visible to its members:

- Custom branding.
- HR / employee-engagement campaigns.
- Internal-meme packs.
- No marketplace listing.

### 14.5 Gifting

Users may gift packs to other users (intra-tenant):

- Sender selects pack → "Send as gift".
- Recipient receives notification with claim link.
- Sender pays; recipient owns.
- Gift limit: 5 per sender per day (anti-spam).

### 14.6 Featured packs

Marketplace home features:

- Editor's pick (curated by oyatie marketplace team).
- Trending (algorithmic; based on installs in last 7 days).
- New (just-released, last 14 days).
- Seasonal (e.g., New Year, holidays per locale).
- Regional (per user locale).

### 14.7 Creator portal

Creators publish packs via `microservices/marketplace` creator portal:

- Pack upload (drag-drop ZIP).
- Manifest validator (in-browser).
- Sticker preview gallery.
- Pricing config.
- Locale strings editor (Fluent + ICU).
- Submit for review.
- Analytics post-launch (installs, retention, revenue).

### 14.8 Revenue payout

Per the payments substrate (post-payments-cert; per ADR-0249 roadmap):

- Monthly payout.
- Threshold: ≥ $50 USD equivalent.
- Tax form collection (W-9 / W-8BEN / equivalent per creator jurisdiction).
- Payouts via marketplace payment provider.

---

## 15. Lottie Animation Support

### 15.1 Why Lottie

Lottie is a JSON-based vector animation format originally from Airbnb. It enables:

- **Tiny file size**: a 50 KB Lottie sticker matches the perceived quality of a 1 MB animated WebP.
- **Lossless scaling**: vector animations render crisp at any size.
- **Skinnable**: themes / colors can be modified at render time.
- **Cross-platform**: official renderers for iOS, Android, web, Windows, Linux, Flutter, React Native.

### 15.2 Lottie spec target

ESRG targets **Lottie format v5.5+** with **dotLottie 2024 spec** (`.lottie` packaging).

dotLottie packages:

- The Lottie JSON (`animation.json`).
- Optional thumbnail.
- Optional secondary animations (e.g., loop variants).
- Manifest (`manifest.json`).

In ZIP form: `.lottie` extension; mime-type `application/zip+dotlottie`.

### 15.3 Lottie players

- **Web**: `@lottiefiles/dotlottie-web` (dotLottie 2024 player) — WASM-backed; renders to canvas; ~80 KB gzipped.
- **iOS**: `lottie-ios` v4.4+ (Airbnb).
- **Android**: `lottie-android` v6.0+ (Airbnb).
- **macOS**: `lottie-ios` (shared with iOS).
- **Windows**: `dotlottie-rs` (Rust binding) or via WebView2 + web player.
- **Linux**: `dotlottie-rs` + Skia.
- **Rust server (validation)**: `dotlottie-rs` (LottieFiles official Rust binding) for pack-build pipeline validation.

### 15.4 Use cases

- **Complex sticker animations**: e.g., a 5-second character dance that would be 4 MB as WebP fits in 200 KB Lottie.
- **Reaction animations**: super-reactions with intricate particle effects.
- **Mascot animations**: tenant-branded mascots that loop on idle.
- **Theme-aware animations**: stickers that adapt color to dark/light mode.

### 15.5 Lottie limitations

- **Photorealistic content not supported**: Lottie is vector; photo-based stickers need WebP/APNG.
- **Complex masks / effects** may render inconsistently across renderers; the marketplace approval pipeline uses a Lottie linter (`dotlottie-rs lint`) to flag risky features.
- **File expressions** are not supported in mobile renderers; rejected by the validator.

### 15.6 Validation

Pack-build pipeline:

1. Parse `.lottie` archive.
2. Validate JSON against Lottie schema.
3. Run `dotlottie-rs lint` for cross-renderer compatibility.
4. Render-test on web/iOS/Android headless renderers; verify visual match.
5. Reject if any test fails.

---

## 16. Bounded Contexts

If the ESRG substrate is extracted into a dedicated future µservice (per ADR-0132's per-microservice flat layout; lane prefix `oya-shared-expression-*` per ADR-0132 governance lane convention), the following bounded contexts are anticipated:

### 16.1 `pack-registry`

Per-tenant registry of installed packs. Citus-sharded Postgres. Bounded context owns:

- Pack install / uninstall.
- Pack version updates.
- Entitlement reconciliation with marketplace.
- Custom emoji index per tenant.

**Inbound contracts**: `pack.install`, `pack.uninstall`, `pack.update.subscribe`.
**Outbound events**: `pack.installed`, `pack.uninstalled`, `pack.deprecated`.

### 16.2 `pack-blob-store`

SeaweedFS bucket per cell; CDN-fronted. Bounded context owns:

- Blob upload (from creator portal).
- Blob retrieval (signed URLs for paid; unsigned for free).
- Garbage collection.
- Replication policy.

**Inbound contracts**: `blob.upload`, `blob.get`, `blob.delete`.
**Outbound events**: `blob.uploaded`, `blob.gc-scheduled`.

### 16.3 `pack-renderer` (client-side)

No server bounded context; client library only. Per platform:

- `oya-expression-web` (npm)
- `oya-expression-ios` (Swift package)
- `oya-expression-android` (Maven)
- `oya-expression-rust` (crate for native desktop apps + Foundry CLI)

The crate naming follows the canonical convention (per `crate-naming-convention.md`).

### 16.4 `gif-proxy`

Tenor + Giphy adapter + caching. Bounded context owns:

- Tenor search.
- Giphy fallback.
- Per-tenant policy enforcement.
- Result caching (24 h Redis cache; 30 day SeaweedFS blob cache).

**Inbound contracts**: `gif.search`, `gif.fetch-binary`.
**Outbound events**: (typically none; read-only adapter).

### 16.5 `custom-emoji-registry`

Per-server + per-tenant + per-user custom emoji storage. Postgres + SeaweedFS.

**Inbound contracts**: `custom_emoji.upload`, `custom_emoji.list`, `custom_emoji.delete`.
**Outbound events**: `custom_emoji.published`, `custom_emoji.deleted`.

### 16.6 `reaction-store`

Per-tenant reaction associations. Citus-sharded Postgres. Bounded context owns:

- Reaction add / remove.
- Reaction aggregation (count + reactor list).
- Cross-surface reaction projection.

**Inbound contracts**: `reaction.add`, `reaction.remove`, `reaction.list`.
**Outbound events**: `reaction.added`, `reaction.removed`.

### 16.7 `moderation-pipeline`

Trust-safety + classifier orchestration. Bounded context owns:

- Pre-publication classifier runs.
- User report queue.
- Bulk takedown actions.
- Appeal workflow.

**Inbound contracts**: `moderation.classify`, `moderation.report`, `moderation.appeal`.
**Outbound events**: `moderation.classified`, `moderation.removed`, `moderation.reinstated`.

### 16.8 Cross-microservice contracts

The substrate's cross-microservice contracts are canonicalized in:

- `microservices/messenger/contracts/proto/expression/*.proto` (gRPC).
- `microservices/messenger/contracts/asyncapi/expression/*.yaml` (Kafka events).
- `microservices/messenger/contracts/openapi/expression.v1.yaml` (REST).

These are versioned (semver) per the event-schema-versioning canonical doc. Breaking changes require a 6-month sunset per the no-silent-regression policy.

---

## 17. User Stories

### 17.1 Story 1: User installs a free sticker pack

**As** a Messenger user
**I want** to install the "Office Reactions" sticker pack
**So that** I can use office-themed stickers in DMs

**Acceptance criteria**:

- User opens Settings → Stickers → Marketplace.
- User browses free packs.
- User clicks "Install" on Office Reactions.
- Pack downloads progressively (manifest + 8 stickers in ≤ 500 ms).
- User opens DM, opens sticker picker, sees Office Reactions tab.
- User taps a sticker; it sends in the message.

### 17.2 Story 2: User uses sticker in a DM

**As** a Messenger user
**I want** to insert a sticker into my message
**So that** I express more than text alone

**Acceptance criteria**:

- User taps the sticker icon in compose.
- Picker opens in < 100 ms.
- User scrolls to find a sticker.
- User taps sticker; it sends.
- Receiver sees the sticker rendered at 200×200 in the message bubble (or 400×400 high-DPI).
- Animated stickers loop.
- Receiver in reduced-motion mode sees static first frame.

### 17.3 Story 3: User creates custom server emoji

**As** a Community server admin
**I want** to upload `:partyparrot:` as a custom emoji
**So that** my community members can use it

**Acceptance criteria**:

- Admin uploads animated WebP via Server Settings → Emojis → Upload.
- Frontend validates size (≤ 1 MB), format (animated WebP).
- Backend re-validates, runs trust-safety classifier.
- Emoji is approved (low classifier score).
- Emoji becomes usable in the server: typing `:party<TAB>` autocompletes to `:partyparrot:`.
- Emoji shows in the reaction picker.

### 17.4 Story 4: User reacts to a message with custom emoji

**As** a Community member
**I want** to react with `:partyparrot:` to a post
**So that** I express celebration in my server's culture

**Acceptance criteria**:

- User hovers over the post.
- Reaction toolbar appears.
- User clicks "+" → emoji picker → `:partyparrot:`.
- Optimistic reaction appears immediately.
- Server confirms within 200 ms.
- Other users see the reaction within 500 ms (via Kafka projection).

### 17.5 Story 5: User searches a GIF and inserts

**As** a Messenger user
**I want** to search "dance" and send a dancing GIF
**So that** I respond playfully

**Acceptance criteria**:

- User taps GIF icon in compose.
- Search bar appears.
- User types "dance" → Tenor results stream in < 300 ms.
- User taps a GIF.
- GIF inserts as a message; renders at 360×270.
- Recipient sees the GIF in their feed; animated WebP fallback to GIF for old clients.

### 17.6 Story 6: User downloads encrypted custom pack to new device

**As** a user with personal sticker packs
**I want** my personal packs to sync to my new phone
**So that** I don't lose my work

**Acceptance criteria**:

- User signs into new device.
- MLS device-key registration completes.
- Personal pack metadata fetched (server-side index).
- Pack blobs (E2EE) fetched and decrypted client-side.
- Packs visible in the picker on new device.
- Tenant pack registry is unaffected (tenant packs are not E2EE; available on any device).

### 17.7 Story 7: User reports inappropriate sticker

**As** a user encountering an offensive sticker
**I want** to report it
**So that** trust-safety can review and remove

**Acceptance criteria**:

- User long-presses the sticker in the picker (or in a message).
- Report option appears.
- User selects reason ("hate symbols") + optional comment.
- Report submitted to trust-safety queue.
- Trust-safety reviews within 24h.
- If validated: sticker removed across all tenants; pack flagged.
- Reporter notified of outcome (default opt-in).

### 17.8 Story 8: Tenant admin restricts sticker categories

**As** a tenant admin in a regulated industry
**I want** to restrict sticker categories to "work" and "celebration"
**So that** my users see only professional content

**Acceptance criteria**:

- Tenant admin opens Tenant Settings → Content → Stickers.
- Admin selects allowed categories: `work`, `celebration`.
- Admin saves.
- Cedar policy updates; users in this tenant only see those categories.
- Existing installed packs in other categories: user retains existing installs but can no longer install new packs in disabled categories.
- Marketplace shows filtered browser to users in this tenant.

### 17.9 Story 9: Marketplace creator publishes a paid pack

**As** a sticker creator
**I want** to publish "Curated Cats" for $2.99
**So that** I earn revenue

**Acceptance criteria**:

- Creator opens marketplace creator portal.
- Creator drags `curated-cats.oyastk` ZIP into uploader.
- Manifest validator runs (in-browser).
- Pack passes validation.
- Creator sets price ($2.99), description, locale strings.
- Creator submits for review.
- Trust-safety reviews within 5 business days.
- Approved → published to marketplace.
- Pack earns revenue; 70% to creator, 30% to oyatie.

### 17.10 Story 10: User subscribes to Premium

**As** a heavy user of stickers
**I want** to subscribe to Premium ($9.99/month)
**So that** I unlock all packs + animated server emojis + super-reactions

**Acceptance criteria**:

- User opens Settings → Premium.
- User selects monthly plan ($9.99).
- User completes payment (via marketplace payment provider).
- Entitlement record minted.
- All Premium packs become installable.
- User's server (where admin) is upgraded to 500 custom emoji limit.
- Super-reactions become available.

### 17.11 Story 11: User adds a super-reaction

**As** a user celebrating a colleague's achievement
**I want** to add a confetti-burst super-reaction
**So that** the celebration is visible to all viewers

**Acceptance criteria**:

- User long-presses the message.
- Quick-bar appears; user swipes to "super-reactions" tab.
- User taps "🎉 Confetti".
- Confetti animation plays for 1.2s overlaying the message thread for all currently-online viewers.
- Reaction stored; future viewers see the static confetti badge.
- Rate-limited: user can do another super-reaction in 60s.

### 17.12 Story 12: User in reduced-motion mode views animated content

**As** a user with motion sensitivity (vestibular disorder)
**I want** animations to not play automatically
**So that** I avoid dizziness

**Acceptance criteria**:

- User has system `prefers-reduced-motion: reduce`.
- All stickers in messages render as static first frame.
- Animated custom emoji render as static.
- GIFs render with a "click to play" overlay; paused by default.
- Super-reactions render as static; no particle effects.
- The user may explicitly tap "play" on any animation to view it.

### 17.13 Story 13: User uses Lottie sticker

**As** a user receiving a Lottie sticker from a creator pack
**I want** smooth, crisp animation at any size
**So that** the sticker looks polished

**Acceptance criteria**:

- Lottie sticker renders via web/native Lottie player.
- File size ~50-200 KB (vs. ~1 MB for WebP equivalent).
- Renders crisply at 200×200, 400×400, 800×800.
- Vector scaling without pixelation.

### 17.14 Story 14: KR user sees regional packs first

**As** a KR-locale user
**I want** to see KR-popular packs first in the marketplace
**So that** I find culturally-relevant content faster

**Acceptance criteria**:

- User's locale = `ko-KR`.
- Marketplace home features "KR Greetings" + KakaoTalk-style packs prominently.
- Locale-pack-defaults.json drives the ordering.
- User can switch to "Global" for international content.

### 17.15 Story 15: User uploads to PHI-tagged tenant

**As** a healthcare-tenant user
**I want** to use stickers in non-PHI conversations
**And not** be able to upload custom stickers (per tenant policy)

**Acceptance criteria**:

- User opens DM (non-PHI-tagged).
- Sticker picker shows oyatie-baseline packs only.
- Custom emoji upload UI is disabled (Cedar deny).
- User attempts to send sticker in PHI-tagged thread: stickers disabled in PHI threads.
- Audit log records every interaction.

### 17.16 Story 16: User exports a Doc with stickers to PDF

**As** a Docs user with sticker-rich comments
**I want** to export to PDF
**So that** I can archive the document

**Acceptance criteria**:

- User clicks "Export → PDF".
- Stickers render as static first frame in PDF.
- Animated GIFs render as static.
- Unicode emoji render via embedded Twemoji SVG (vector; scales well in PDF).
- Reactions appear as text tally next to comments: "👍 5 ❤️ 3".

### 17.17 Story 17: User gifts a pack

**As** a Premium subscriber
**I want** to gift "Curated Cats" to a colleague
**So that** I share something fun

**Acceptance criteria**:

- User opens pack in marketplace.
- "Send as gift" option appears.
- User selects recipient (must be intra-tenant).
- Payment confirmation.
- Recipient receives notification with claim link.
- Recipient claims; pack installs.
- Gift limit: 5 per day.

### 17.18 Story 18: Tenant admin imports legacy LINE pack

**As** a tenant migrating from LINE
**I want** to import my LINE pack catalog (where licensing permits)
**So that** my users retain familiar content

**Acceptance criteria**:

- Tenant admin uploads LINE-export ZIP.
- ESRG import tool maps LINE format → oyatie pack format.
- Trust-safety + licensing re-validation.
- Approved packs installed tenant-wide.
- Rejected packs flagged for admin review.

### 17.19 Story 19: User uses sticker prediction from emoji

**As** a Messenger user
**I want** the keyboard to suggest stickers when I type emoji
**So that** I discover relevant stickers faster

**Acceptance criteria**:

- User types 🎉 in compose.
- Sticker suggestions appear below keyboard.
- Suggestions sourced from `emoji_associations` of installed packs.
- Tap to insert sticker.
- Performance: ≤ 50 ms after emoji insert.

### 17.20 Story 20: Auto-moderation flags spam emoji upload

**As** a tenant admin
**I want** spam-bulk custom emoji uploads to be auto-flagged
**So that** my server isn't overwhelmed

**Acceptance criteria**:

- A new user uploads 100 emojis in 10 minutes.
- Auto-moderation rate-limit triggers.
- Subsequent uploads queued for admin approval.
- Admin notified; reviews and bulk-approves or bulk-rejects.

### 17.21 Story 21: User requests GDPR Article 17 erasure

**As** a user invoking right-to-erasure
**I want** my custom emoji uploads and personal packs deleted
**And** my past reactions anonymized

**Acceptance criteria**:

- User submits erasure request via Settings → Privacy.
- ESRG identifies all user-owned emoji, packs, reactions.
- Custom emoji deleted (tombstoned 30 days; GC'd).
- Personal packs deleted (E2EE blobs unrecoverable post-delete).
- Past reactions: sender_id anonymized to sentinel; reactor-list no longer shows the user.
- Audit log records the erasure.

### 17.22 Story 22: Cross-surface reaction (Docs → Notification → Mail)

**As** a user who reacted in a Doc
**I want** the email notification of my reaction to render correctly
**So that** non-Docs viewers see what I reacted

**Acceptance criteria**:

- User reacts to a Docs comment with 🚀.
- Notification email sent.
- Email body inlines the rocket emoji as Unicode (recipient client renders).
- Custom emoji notifications inline as image attachment with alt text.
- Animated custom emoji renders as static first frame in email.

---

## 18. References

### 18.1 Standards + specs (2024-2026 era)

1. **Unicode 16.0 emoji** — released 2024-09; 7 new glyphs (face with bags under eyes, fingerprint, root vegetable, leafless tree, splatter, harp, shovel). [Unicode emoji 16.0 release notes](https://unicode.org/emoji/charts-16.0/emoji-released.html)
2. **Unicode CLDR v46+** — emoji annotations + 50+ locale translations.
3. **Unicode UTS #51** — Unicode Emoji standard.
4. **Twemoji v15+** — Twitter open-source emoji set; CC-BY-4.0; the actively-maintained `jdecked/twemoji` fork as of 2024.
5. **Noto Emoji** (Google) — SIL Open Font License 1.1.
6. **WebP animation** — Google; [WebP container spec](https://developers.google.com/speed/webp/docs/riff_container).
7. **APNG spec** — animated PNG; supported by all modern browsers since 2017.
8. **Lottie spec** — vector animation JSON; current spec v5.5+.
9. **dotLottie 2024 spec** — LottieFiles; .lottie packaging.
10. **WCAG 2.2** — accessibility; published 2023-10.

### 18.2 Industry references

11. **Discord custom emoji architecture** — Discord engineering blog (2019–2024); per-server emoji + Nitro animated emoji.
12. **Slack emoji + reaction product docs** — Slack help center; reaction-centric UX patterns.
13. **KakaoTalk sticker creator** (KR) — Kakao Emoticon Studio; creator-economy precedent.
14. **LINE sticker shop** (JP) — Line Creators Market; the original sticker-marketplace.
15. **Apple Sticker Pack Guidelines 2024** — Apple HIG; Messages.app sticker drawer UX.
16. **Tenor API docs** — Google Tenor v2 API; `https://developers.google.com/tenor/guides/quickstart`.
17. **Giphy API docs** — Meta-adjacent; `https://developers.giphy.com/`.
18. **Telegram Stickers API** — Telegram's .tgs Lottie-based animated sticker format.
19. **WhatsApp Sticker API** — Meta WhatsApp Business Platform sticker pack format (WebP).
20. **Microsoft Teams emoji + reaction** — Microsoft Learn; tenant-policy controls.
21. **Mozilla Project Fluent** — localization framework.
22. **ICU MessageFormat** — message localization standard.

### 18.3 Internal references

23. `microservices/messenger/` PRD + contracts + IPs.
24. `microservices/community/` PRD + contracts.
25. `microservices/marketplace/` (forthcoming) PRD + ADRs.
26. `microservices/plugin-app-store/` PRD + packs.
27. `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` — communication patterns.
28. `docs/decisions/ADR-0700-ci-admission-live-apex.md` — Cedar policy gating.
29. `docs/decisions/ADR-0708-platform-foundations-live-apex.md` — compliance pack tiers.
30. `docs/standards/crate-naming-convention.md` — `oya-shared-expression-*` naming.
31. `docs/standards/api-design.md` — REST + gRPC + AsyncAPI canonical conventions.
32. `docs/standards/i18n-canonical.md` — internationalization.
33. `docs/standards/a11y-canonical.md` — accessibility.
34. `docs/standards/event-schema-versioning-canonical.md` — event evolution.
35. `docs/standards/ux-best-practices.md` — (parallel work; sticker picker UX adheres).

### 18.4 Cited libraries

36. SDWebImage (iOS / macOS) — Apache-2.0.
37. SDWebImageWebPCoder — Apache-2.0.
38. Coil (Android) — Apache-2.0.
39. Glide (Android) — BSD-3-Clause.
40. lottie-ios / lottie-android (Airbnb) — Apache-2.0.
41. @lottiefiles/dotlottie-web (LottieFiles) — MIT.
42. dotlottie-rs (LottieFiles Rust) — MIT.
43. @twemoji/parser — MIT.
44. emoji-mart (web emoji picker reference) — MIT.

---

## 19. Appendices

### Appendix A: Pack-build CLI

The `oya-stickers` CLI (Rust binary; ships in `tools/`) provides:

```
oya-stickers init <pack-name>             # scaffold a new pack
oya-stickers add <sticker-file>           # add a sticker to current pack
oya-stickers validate                     # validate manifest + assets
oya-stickers build                        # build .oyastk archive
oya-stickers sign --key <ed25519-key>     # sign for marketplace
oya-stickers publish --marketplace-url    # upload to marketplace
oya-stickers lint                         # cross-renderer Lottie lint
```

### Appendix B: Picker UX wireframes (textual)

**Desktop emoji + sticker picker** (640×480 popover):

```
┌──────────────────────────────────────────────────────────────┐
│  🔍  Search emoji and stickers...                         ⚙️ │
├──────────────────────────────────────────────────────────────┤
│  Recent ⏱ │ Smiles 😄 │ People 🧑 │ Animals 🐶 │ Food 🍔  ▶ │
│  Stickers │ Office │ Cats │ KR-Greet │ More ▶                │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│   😀  😃  😄  😁  😆  😅  🤣  😂  🙂  😉  😊  😇  🥰  😍   │
│   🤩  😘  😗  ☺️  😚  😙  🥲  😋  😛  😜  🤪  😝  🤑  🤗   │
│   🤭  🤫  🤔  🤐  🤨  😐  😑  😶  😏  😒  🙄  😬  🤥  😌   │
│   ...                                                        │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│  Skin tone:  ▢  🏻  🏼  🏽  🏾  🏿                            │
└──────────────────────────────────────────────────────────────┘
```

**Mobile sticker drawer** (full-width bottom-sheet):

```
┌──────────────────────────────────────────────────────────────┐
│  ━━━━━━━                                                    │
│  Stickers                                              ✕     │
├──────────────────────────────────────────────────────────────┤
│  🔍 Search                                                   │
├──────────────────────────────────────────────────────────────┤
│  ⏱  📦  💼  🐱  🇰🇷  🍔  🎉  ➕                              │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│   [sticker]  [sticker]  [sticker]  [sticker]                 │
│   [sticker]  [sticker]  [sticker]  [sticker]                 │
│   [sticker]  [sticker]  [sticker]  [sticker]                 │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│  📦 Office Reactions · @oyatie-originals          ⓘ          │
└──────────────────────────────────────────────────────────────┘
```

### Appendix C: Error handling

| Error | UX |
|---|---|
| Pack download timeout | Retry banner; user can manually retry; partial pack remains usable |
| Sticker render fails | Show fallback alt-text + "couldn't load" placeholder |
| Custom emoji upload too large | Inline error: "Max 1 MB for animated; 256 KB for static" |
| Reaction add network error | Show optimistic; roll back with shake; banner "couldn't react" |
| GIF search empty results | "No results — try a different search" |
| Tenor API rate-limited | Fall back to Giphy; if both fail, "GIF search temporarily unavailable" |
| Cedar deny on install | "Your tenant admin has restricted this pack" |
| Pack signature invalid | Block install; alert security team |
| Trust-safety auto-reject | Show "this sticker is unavailable"; route to creator for appeal |

### Appendix D: Migration from competitor systems

For tenants migrating from Slack, Discord, KakaoTalk, or LINE:

- **Slack import**: per-workspace custom emoji exported via Slack admin tools; oyatie ingestion CLI (`oya-stickers import slack-export.zip`) recreates per-server custom emoji.
- **Discord import**: per-server custom emoji; same CLI tool.
- **KakaoTalk migration**: limited; users may need to re-purchase due to KakaoTalk DRM restrictions.
- **LINE migration**: limited; LINE sticker DRM restricts re-distribution.

### Appendix E: Test matrix

Minimum test coverage for the ESRG substrate:

| Test | Surface |
|---|---|
| Unit: pack manifest validator | `oya-stickers` CLI |
| Unit: Cedar policy evaluator (sticker.pack.install) | substrate |
| Unit: WebP decode | client SDKs |
| Unit: Lottie linter | `oya-stickers` CLI |
| Integration: pack install via marketplace → registry → blob store | end-to-end |
| Integration: reaction add → projection across surfaces | end-to-end |
| Integration: GIF search via Tenor → result rendering | end-to-end |
| Integration: custom emoji upload → approval queue → publication | end-to-end |
| Performance: sticker picker open p95 ≤ 100 ms | client SDKs |
| Performance: pack download progressive p95 ≤ 500 ms phase 1 | end-to-end |
| Accessibility: screen-reader announces all stickers + emoji | client SDKs |
| Compliance: GDPR erasure removes user-owned packs + reactions | end-to-end |
| Cross-platform: same sticker pack renders identically on web/iOS/Android/macOS/Windows/Linux | visual diff |
| Localization: every emoji + sticker has en-US + ko-KR + ja-JP names | data |
| Security: pack signature verification rejects tampered archives | substrate |

### Appendix F: Roadmap

| Version | Scope |
|---|---|
| v1.0 (this spec) | Unicode 16, Twemoji default, sticker packs, reactions, GIF (Tenor + Giphy), custom emoji per-server / per-tenant, marketplace integration, Lottie, all surfaces |
| v1.1 | Meet live-overlay reactions (WebRTC datachannel); WinUI 3 + GTK4 / Qt6 native |
| v1.2 | Recordings post-call reaction timeline; super-reaction creator portal |
| v1.3 | Sound stickers (audio-bearing stickers, Discord-Soundboard parity) |
| v2.0 | AI-generated stickers (Intelligence Substrate); 3-D / AR stickers (Memoji parity) |
| v2.1 | Sticker minigames / interactive stickers |

### Appendix G: Open questions

1. **Closed-store imports** — LINE / KakaoTalk packs have DRM; we may not legally redistribute. Decision: scope as **import-by-customer-license** (users present their original purchase as proof); pending legal review per ADR-0249.
2. **Animated AVIF** — emerging format; promising compression vs. WebP. Decision: track adoption; reconsider for v1.2 once browser support is universal.
3. **Per-user emoji preference cross-tenant** — what if a user in tenant-A prefers Apple emoji but tenant-A forbids it? Resolution: user-pref subordinates to tenant-pref; documented in [§2.4](#24-per-tenant-override).
4. **Pack version conflicts** — user has v1.0 of pack X; new v2.0 deprecates 5 stickers used in past messages. Decision: old stickers continue to render from v1.0 blobs (tombstoned 90 days); new sends use v2.0.
5. **Trust-safety automation vs. human review** — auto-approval thresholds; pending separate ADR per ADR-0251 trust-safety substrate work.

### Appendix H: SLOs

| SLO | Target | Window |
|---|---|---|
| Sticker picker availability | 99.95% | 30d |
| Marketplace pack browse latency p95 | ≤ 400 ms | 30d |
| Pack download success rate | ≥ 99.5% | 30d |
| Reaction add success rate | ≥ 99.9% | 30d |
| GIF search availability (Tenor primary) | ≥ 99.0% | 30d |
| Custom emoji upload success rate | ≥ 99.5% | 30d |
| Trust-safety review SLA (high-severity) | ≤ 4 hours | 30d |
| Trust-safety review SLA (normal) | ≤ 24 hours | 30d |
| Erasure (GDPR Art 17) completion | ≤ 30 days | per request |

SLO documents authored under `microservices/messenger/slos/` per ADR-0130 / ADR-0131.

### Appendix I: Operational runbooks

Located at `microservices/messenger/runbooks/`:

- `sticker-pack-takedown.md` — bulk pack removal.
- `tenor-api-failure.md` — fallback to Giphy + self-hosted.
- `marketplace-payment-reconciliation.md` — entitlement sync.
- `custom-emoji-spam-burst.md` — auto-mod tuning.
- `pack-blob-corruption-recovery.md` — SeaweedFS recovery.

### Appendix J: Telemetry

Per the observability substrate (ADR-0130/0131):

- **Counters**: stickers sent (by pack, surface), GIFs searched, reactions added (by surface + payload-kind), custom emoji uploads, super-reactions used.
- **Histograms**: picker-open latency, sticker-render time, pack-download time, GIF-search latency.
- **Gauges**: active installs per pack, per-tenant emoji count, marketplace MAU.
- **Tracing**: sticker insertion end-to-end (compose → send → projection → read).

All metrics tagged per tenant + per cell + per surface (matching `microservices/messenger/dashboards/` conventions).

### Appendix K: Security threat model

| Threat | Mitigation |
|---|---|
| Malicious pack with code execution | Pack archives are inert (images + JSON only); manifest + blob-content validation; no executable formats accepted |
| XSS via custom SVG emoji | Server sanitizes SVG (DOMPurify equivalent in Rust); script tags stripped |
| Stego payload in pack | Blob re-encoded at upload; randomized re-compression destroys most stego |
| Pack signature forgery | Ed25519 signature per pack; public key pinned to creator account |
| Tenor / Giphy poisoning | Proxy filters by tenant policy; classifier re-checks high-risk content |
| Rate-limit bypass | Per-user + per-tenant rate limits enforced at API gateway |
| CSAM upload | Pre-publication classifier (Intelligence Substrate); auto-block; auto-report per legal jurisdiction |
| Copyright infringement | Fingerprint match against known IP database; DMCA workflow |
| Tenant-policy bypass | Cedar deny-by-default; admin-set policies enforced at every read + write |
| Emoji-name impersonation | Reserved prefixes (`oya_`, `system_`, `unicode_`) cannot be claimed by user-uploads |

### Appendix L: Capacity planning

Per the capacity model (`microservices/messenger/capacity-model.md`):

- **Storage per active tenant** (1000 users, 50 packs installed): ~5 GB.
- **CDN bandwidth per MAU**: ~50 MB/month (reactions + stickers + occasional GIF; dominated by GIFs).
- **DB IOPS per active reaction-event**: ~3 (insert + projection + denorm aggregate).
- **Marketplace browse QPS at peak**: ~10K/s (estimated for 10M MAU; cached at edge).

### Appendix M: Decommissioning

If a pack creator decommissions a pack:

- **Existing installs** remain functional for 12 months.
- **New installs** disabled immediately.
- **Past messages** referencing pack stickers: continue to render for the 12-month grace period; after, fallback to alt-text + placeholder.
- **Tenant admin** notified 90 days before final GC.

---

**END OF DOCUMENT**

Document checksum + version: v1.0 — 2026-05-20 — first canonical draft (lane: standards; surface: cross-microservice). Maintained by the messenger working group with cross-microservice review.
