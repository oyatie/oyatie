---
id: ADR-0185
status: Superseded
deciders: council-architecture, axis-workflow-studio, axis-mobile, axis-desktop
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
amended_by: [ADR-0632]
related: [ADR-0064, ADR-0131, ADR-0145, ADR-0148, ADR-0182, ADR-0183, ADR-0184, ADR-0186]
related_specs:
  - /specs/microservices/manifest-schema.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.

# ADR-0185 — Workflow Studio client stack: per-surface native rendering; OpenAPI contract is the cross-ecosystem unifier

## Status

Accepted (2026-05-18). Adopts a **per-surface native rendering** strategy for Workflow Studio clients (and the same template applies to other oyatie end-user products). Each platform target gets its idiomatic native stack; the cross-ecosystem unifier is **OpenAPI 3.2.0 contract-first codegen**, not a shared UI framework or shared business-logic layer.

## Status note (user directive 2026-05-18)

User directive: *"Native is best for everything where possible."* This ADR enforces that directive across every supported platform.

## ADR-0632 product-protocol reconciliation

Workflow Studio and other clients call the public HTTPS REST gateway documented by OpenAPI 3.2.0 and may consume signed/versioned webhooks, AsyncAPI/CloudEvents events, SSE, or WebSocket where appropriate. Public GraphQL, gRPC, gRPC-Web, and Connect are forbidden. Protobuf is limited to internal-only gRPC/proto3 over HTTP/2 behind the gateway.

## Context

Workflow Studio is oyatie's flagship n8n-class first-hero product. It targets:

- Web (browser) — primary editing surface for desktop power users.
- iOS / iPadOS / macOS / watchOS / visionOS — Apple ecosystem.
- Android — non-Apple mobile + tablet.
- Windows desktop — Microsoft ecosystem.
- Linux desktop — open-source ecosystem.

The hyperscaler bar:

- **Quality** — best native UX per platform; no lowest-common-denominator UI.
- **Maintainability** — codegen unifies API contract surface; per-ecosystem shared layers minimize per-target boilerplate.
- **Scalability** — five client teams can ship independently with strong ecosystem separation.
- **Integration** — every client speaks the same OpenAPI 3.2.0 contract; backend is Rust per ADR-0120; codegen consumes the same OpenAPI source.

Anti-patterns this ADR forecloses:

1. **One UI framework everywhere** (Flutter, React Native, Compose Multiplatform UI) — produces non-idiomatic UX per platform; lowest-common-denominator UI quality.
2. **WebView wrappers as primary native clients** (Electron, Tauri for Windows/macOS) — not truly native; slower; non-idiomatic UX; rejected by Apple App Store guidelines for non-trivial apps.
3. **KMP UI across Apple + Android** — Swift idioms (Result builders, async/await with structured concurrency, actor model) don't round-trip through KMP cleanly; pure Swift on Apple is the modern Apple-native bar.

## Decision

Oyatie adopts the following per-surface client matrix. Each surface is native; each ecosystem shares logic via its own idiomatic shared-layer; the cross-ecosystem unifier is the OpenAPI 3.2.0 contract.

### Per-surface client matrix

| Surface | Stack | API client codegen | Ecosystem-shared layer | Status |
|---|---|---|---|---|
| Web (Phase 1) | **SvelteKit 2.55 + Svelte 5.55 (runes) + Vite 8.0 + TypeScript 6.0** | `openapi-typescript` + `openapi-fetch` | `clients/web-sveltekit/packages/shared-ts/` | active |
| Web (Phase 2) | **Leptos 0.8.x** (SSR + hydration; fine-grained signals reactivity) | **`progenitor`** (Rust openapi codegen) | `crates/oya-client-shared-rust/` (Rust workspace crate) | scaffold now, ship at Leptos 1.0 |
| Apple (iOS/iPadOS/macOS/watchOS/visionOS) | **Swift 6.3 + SwiftUI ONLY** | Apple official **`swift-openapi-generator` 1.11.1** | `clients/apple/shared-swift/` (Swift Package consumed by all Apple targets via SPM) | active |
| Android | **Kotlin 2.3 + Jetpack Compose** | **OpenAPI Generator Kotlin** (Ktor client target) | `clients/android/shared-kotlin/` (KMP module — Android scope only) | active |
| Windows desktop | **WinUI 3 + .NET 10 LTS (C#)** via Windows App SDK 1.8 | Microsoft **Kiota 1.31.1** | `clients/windows/Shared.csproj` (.NET class library) | active |
| Linux desktop | **GTK 4 + gtk-rs 0.11.3 + libadwaita 1.8 (Rust)** | **`progenitor`** (shared with Leptos Phase 2) | `crates/oya-client-shared-rust/` (shared with Leptos) | active |

### Web stack lifecycle (Phase 1 → Phase 2; sequential, NOT parallel)

The web stack runs **SvelteKit-now → Leptos-future**, not dual-parallel.

- **Phase 1** (now → Leptos 1.0, ETA mid-2026): SvelteKit 2.55 is the sole web stack. Ship Workflow Studio editor, Ops Portal, web surfaces, marketing/docs all in SvelteKit. Phase 1 active 2026-05-18.
- **Phase 2 trigger conditions** (ALL must hold):
  - Leptos 1.0 has shipped a stable release (currently 0.8.x; 1.0 outlook documented in Leptos roadmap).
  - Leptos 1.0 has SSR + islands routing + auth-aware reactivity ergonomics matching SvelteKit.
  - Two pilot internal surfaces have shipped on Leptos and run for >= 60 days without rollback.
- **Phase 2 migration**: Begin migration surface-by-surface. New surfaces in Leptos; existing surfaces migrate when they next have major UX work. No big-bang rewrite. ONE web stack at a time per surface — never SvelteKit + Leptos for the same surface in parallel.
- **Phase 3** (Leptos parity): SvelteKit retired surface-by-surface. Final SvelteKit shutdown ADR when the last surface migrates.

### Cross-platform shared business logic

There is no single cross-ecosystem shared-logic library. Each ecosystem owns its own shared layer:

- **Apple** (Swift): `clients/apple/shared-swift/` — Swift Package consumed by every Apple target (iOS, iPadOS, macOS, watchOS, visionOS) via SPM. Models, validators, repository layer — all idiomatic Swift.
- **Android** (Kotlin): `clients/android/shared-kotlin/` — KMP module scope is Android targets + optional JVM/server reuse only. **Not** consumed by Apple targets.
- **Web TypeScript**: `clients/web-sveltekit/packages/shared-ts/` — TypeScript package consumed by SvelteKit.
- **Web Rust (Phase 2)**: `crates/oya-client-shared-rust/` — Rust workspace crate, WASM-compiled for Leptos consumption.
- **Linux GTK** (Rust): consumes the same `crates/oya-client-shared-rust/` crate (shared with Leptos Phase 2).
- **Windows** (.NET): `clients/windows/Shared.csproj` — .NET class library.

**Cross-ecosystem unifier**: OpenAPI 3.2.0 contract. Every codegen produces identically-shaped DTOs in each ecosystem's idiom. The contract IS the cross-ecosystem shared layer; per-ecosystem shared libs add only the ecosystem-idiomatic layer above the codegen output.

### Cross-surface design tokens

Shared design tokens authored in JSON (Style Dictionary format) at `clients/design-tokens/` compile to:

- Tailwind config (SvelteKit Phase 1).
- Leptos CSS modules (Phase 2).
- SwiftUI Color / Font extensions (Apple).
- Android Jetpack Compose theme (Material 3 token mapping).
- WinUI 3 ResourceDictionary (XAML resource bundles).
- Linux GTK CSS (libadwaita-aware token bindings).

Per-platform token compilation runs in CI; drift between source tokens and compiled outputs fails the build.

### Repository layout

```
microservices/workflow-studio/clients/
├── web-sveltekit/                           # Phase 1
│   ├── packages/shared-ts/
│   └── apps/<surface>/
├── web-leptos/                              # Phase 2 scaffold
│   └── Cargo.toml (workspace member)
├── apple/                                   # Swift + SwiftUI
│   ├── shared-swift/                        # SPM shared Swift package
│   ├── ios-app/
│   ├── ipados-app/
│   ├── macos-app/
│   ├── watchos-app/
│   ├── visionos-app/
│   └── Package.swift                        # SPM workspace root
├── android/                                 # Kotlin + Compose
│   ├── shared-kotlin/                       # KMP module — Android target scope only
│   └── app/                                 # Compose UI
├── windows/                                 # WinUI 3 + .NET 10 LTS
│   ├── Shared.csproj
│   └── WorkflowStudio.WinUI3/
├── linux/                                   # GTK 4 + gtk-rs (Rust)
│   ├── Cargo.toml                           # workspace member
│   ├── src/
│   │   ├── main.rs                          # gtk4 Application entrypoint
│   │   ├── ui/                              # libadwaita widgets
│   │   └── viewmodel/                       # uses crates/oya-client-shared-rust
│   └── data/                                # .desktop, AppStream metainfo, .gschema.xml
└── design-tokens/                           # JSON tokens → all platforms via Style Dictionary
```

Each `clients/*/` directory ships a real **`client-manifest.json`** declaring stack version + codegen recipe + dependency lockfile path + ecosystem-shared-layer path. Every client target ships a real "hello world" entrypoint that builds + runs a smoke test per platform — no stubs.

### Linux packaging

The Linux native client is packaged via:

- **Flatpak** (Flathub primary distribution) using `org.gnome.Platform` GNOME 49 / 50 runtime (GNOME 48 EOL 2026-03; track GNOME release cadence per ADR-0098).
- **.deb** (Ubuntu/Debian) and **.rpm** (Fedora/RHEL) for distro-native installs.
- **AppImage** for universal/portable installs.
- **From-source `cargo install`** for power users.
- Snap is optional; not canonical.

## Alternatives considered

### (a) React + Vite + Radix + Tailwind for web — REJECTED

- **Pros:** widest ecosystem; biggest hiring pool; vast component library universe.
- **Cons:** locks frontend to React ecosystem; not aligned with Rust-primary endgame; "yet another framework choice" indifference; React's reactive model and useEffect lifecycle are weaker per-unit-of-DX than Svelte 5 runes or Leptos signals; React's bundle size remains larger.
- **Rejected**: locks ecosystem direction against the Rust-primary endgame.

### (b) Next.js — REJECTED

- **Pros:** mature; rich plugin ecosystem.
- **Cons:** Vercel ecosystem coupling; Vercel's commercial incentives skew Next.js roadmap; locks self-hosting story.
- **Rejected**: vendor-soft-lock.

### (c) Solid + SolidStart — REJECTED

- **Pros:** signals-reactivity model (similar to Leptos); fast.
- **Cons:** smaller ecosystem than Svelte; Leptos covers signals-reactivity in Rust which aligns with oyatie's Rust-primary endgame.
- **Rejected**: redundant with Leptos for signals; smaller ecosystem than Svelte for Phase 1.

### (d) Vue + Nuxt — REJECTED

- **Pros:** mature; large community.
- **Cons:** Vue's evolution velocity (Vue 3 + composition API + script setup) less predictable than Svelte 5's settled runes model; smaller component library ecosystem than React.
- **Rejected**: less predictable evolution; smaller ecosystem than React (and we already rejected React).

### (e) Electron / Tauri for primary native desktop — REJECTED FOR PRIMARY

- **Pros:** single web codebase across desktop platforms.
- **Cons:** not truly native; WebView per platform (Tauri uses WebKit/WebView2) means non-idiomatic UX vs native toolkits; rejected by Apple App Store guidelines for non-trivial apps; user directive "native is best for everything where possible".
- **Rejected as primary**: native toolkits per platform are the bar.

### (f) Flutter for everything — REJECTED

- **Pros:** single Dart codebase across mobile + desktop + web.
- **Cons:** Dart-only stack mismatches the Rust/Swift/Kotlin team skill mix; cross-platform UI lowest-common-denominator; weaker native ergonomics per platform.
- **Rejected**: language mismatch + LCD UI.

### (g) React Native — REJECTED

- **Pros:** code-share with web React (rejected anyway).
- **Cons:** JS bridge overhead; native UI is the bar; performance and ergonomics consistently behind native Swift/Kotlin.
- **Rejected**: not truly native.

### (h) KMP shared across Apple + Android — REJECTED

- **Pros:** code-share between iOS and Android.
- **Cons:** Swift/klib interop adds ObjC header generation friction; Swift idioms (Result builders, async/await with structured concurrency, actor model) don't round-trip through KMP cleanly; pure Swift on Apple is the modern Apple-native bar; Compose Multiplatform iOS reached stable in 2025 (1.8) and is currently at 1.11.0 but produces non-idiomatic iOS UX vs SwiftUI.
- **Rejected**: SwiftUI is the canonical Apple UI; KMP scope shrinks to Android-only.

### (i) Compose Multiplatform on iOS — REJECTED

- **Pros:** code-share with Android Compose; iOS stable since 1.8.
- **Cons:** even with stability, produces non-idiomatic iOS UX (animations, navigation, controls); Apple's design language evolves via WWDC and SwiftUI ships those changes first; SwiftUI is canonical for Apple.
- **Rejected**: non-idiomatic iOS UX.

### (j) PWA-only for Linux — REJECTED

- **Pros:** zero native Linux complexity; reuse web.
- **Cons:** user directive "native is best for everything where possible"; Linux deserves a real native client; PWA misses native AT-SPI accessibility, system-tray integration, Wayland-first ergonomics, .desktop file metadata.
- **Rejected**: native Linux is required by directive.

### (k) Qt 6 (qt-rs / PyQt) for Linux — REJECTED

- **Pros:** mature; cross-platform.
- **Cons:** Qt licensing fragmentation (LGPL + commercial split); GTK is the GNOME default + better Wayland support + simpler licensing (LGPL).
- **Rejected**: licensing + GTK ecosystem alignment.

### (l) libcosmic for Linux — REJECTED

- **Pros:** Rust-native; modern Cosmic look-and-feel.
- **Cons:** Cosmic adoption is small (System76 Pop!_OS only as of 2026-05); ecosystem young; revisit when Cosmic 2.0 GA matures.
- **Rejected for now**: ecosystem maturity.

### (m) Slint / Iced for Linux — REJECTED

- **Pros:** Rust-native GUI.
- **Cons:** Slint smaller ecosystem; Iced uses own widget toolkit (not native GTK), produces non-native UX, mismatches user directive.
- **Rejected**: non-native UX (Iced); ecosystem maturity (Slint).

### (n) **CHOSEN: per-surface native, OpenAPI codegen unifier**

- **Pros:**
  - Best native UX per platform; user directive honored.
  - OpenAPI 3.2.0 contract-first codegen unifies all clients on the same backend types.
  - Per-ecosystem shared layers minimize boilerplate without forcing LCD UI.
  - Rust workspace crate `oya-client-shared-rust` is shared between Leptos (Phase 2) + Linux GTK, reinforcing the Rust-primary endgame.
  - Five client stacks teach hiring focus on platform specialists, not jack-of-all-trades framework knowledge.
- **Cons:**
  - Five client stacks to maintain. Mitigated by per-surface team ownership; OpenAPI codegen unifier; per-ecosystem shared-layer kits.
  - iOS and Android share NOTHING at the language layer (explicit decision). Mitigated by the OpenAPI contract layer carrying the cross-ecosystem unification.
  - Hiring complexity: Swift, Kotlin, C#, TypeScript, Rust skills on the client team. Mitigated by platform-specialist hiring.
- **Accepted**.

## Consequences

### Positive

1. **Best native UX per platform.** No LCD framework forcing platform inconsistencies.
2. **Pure-Swift Apple stack is the same shape used by Apple's own first-party apps** (Apple Intelligence, Vision Pro apps, App Store) — best long-term ergonomics + hiring fit.
3. **OpenAPI 3.2.0 contract-first codegen unifies all clients** on the same backend types. The contract IS the cross-ecosystem shared layer.
4. **Rust workspace crate `oya-client-shared-rust` reinforces Rust-primary direction**, shared by Leptos Phase 2 + Linux GTK.
5. **Cross-surface design tokens** (Style Dictionary JSON) compile to platform-native resources; one source of design truth.
6. **Independent shipping per surface** — five client teams ship independently; OpenAPI contract is the integration point.
7. **Linux deserves native (GTK + Wayland + AT-SPI)** — honors directive; better than PWA on a Linux desktop.

### Negative

1. **Five client stacks.** Mitigation: per-surface ownership; codegen unifier; per-ecosystem shared kits.
2. **iOS + Android share NOTHING at language layer** — explicit decision; share only at OpenAPI contract layer. Mitigation: cross-ecosystem unifier is the contract; per-ecosystem shared kits cover the rest.
3. **Hiring complexity** — Swift / Kotlin / C# / TypeScript / Rust. Mitigation: per-platform specialist hiring; team ownership.
4. **Web Phase 1 → Phase 2 migration requires Leptos 1.0 to reach SvelteKit parity.** Mitigation: Phase 2 trigger conditions documented; no migration before they hold; surface-by-surface migration without big-bang rewrite.

### In-house roadmap

Per user directive 2026-05-18 (in-house-stack policy), every client stack component classifies as follows:

| Component | Classification | Rationale | In-house Phase 2 plan |
|---|---|---|---|
| **SvelteKit 2.55** | KEEP (MIT; Vercel-stewarded but governance is community-led; State-of-JS 93% satisfaction) | Web meta-framework standard for Phase 1; runes-first Svelte 5 ergonomics. | Phase 1 only; **Phase 2 web stack shift is to Leptos** (Rust-primary in-house direction). |
| **Svelte 5 / runes** | KEEP (MIT) | Settled reactive primitives. | Migrates to Leptos in Phase 2. |
| **Vite 8.0 (Rolldown)** | KEEP (MIT; Rolldown is Rust-native) | THE standard JS build tool; Vite 8 + Rolldown is itself a Rust-primary shift inside the JS ecosystem. | Phase 2 web uses Leptos workspace (cargo); Vite retired surface-by-surface as Leptos migration completes. |
| **TypeScript 6.0** | KEEP (Apache 2.0; Microsoft + community) | THE JS-ecosystem typed-superset standard. TypeScript 7.0 Beta (Go-rewrite) coming — track per ADR-0098 LTS rotation. | None planned (TypeScript is upstream-stewarded). |
| **Leptos 0.8.x → 1.0** | KEEP (MIT/Apache-2.0; community) | Rust web framework with signals reactivity; aligns with the Rust-primary endgame. **This Phase 2 migration IS oyatie's in-house web shift** — moving the web tier into the Rust workspace that the rest of oyatie's infra speaks. | None — Leptos IS the in-house Phase 2 direction. The `oya-client-shared-rust` workspace crate (shared with Linux GTK client) reinforces Rust-primary. |
| **Swift 6.3 + SwiftUI** | KEEP (Apple platform standard) | THE canonical Apple-platform native stack; what Apple's first-party apps use. | None planned. Apple-platform-native is always KEEP. |
| **`swift-openapi-generator` 1.11.1** | KEEP (Apache 2.0; Apple-published) | Apple's official OpenAPI codegen. | None planned. |
| **Kotlin 2.3.0** | KEEP (Apache 2.0; JetBrains + Google-backed; Android platform standard) | THE Android-native language. | None planned. |
| **Jetpack Compose** | KEEP (Apache 2.0; Google; Android platform standard) | THE Android-native UI framework. | None planned. |
| **Kotlin Multiplatform (Android-scope)** | KEEP (Apache 2.0; JetBrains) | Android shared-logic module; KMP iOS rejected here per ADR-0185. | None planned. |
| **WinUI 3 / Windows App SDK 1.8** | KEEP (MIT; Microsoft; Windows platform standard) | THE Windows-native UI framework. | None planned. |
| **.NET 10 LTS** | KEEP (MIT; .NET Foundation) | LTS through 2028; Windows-native. | None planned. |
| **Microsoft Kiota 1.31.1** | KEEP (MIT; Microsoft) | Microsoft's official OpenAPI codegen. | None planned. |
| **gtk4-rs 0.11.3** | KEEP (MIT; GNOME Foundation / gtk-rs project) | THE Rust-native GTK 4 bindings. Reinforces Rust-primary on Linux. | None planned. |
| **libadwaita 1.8** | KEEP (LGPL-2.1+; GNOME Foundation) | THE GNOME-native widget library. | None planned. |
| **GNOME / Flatpak runtime** | KEEP (open standard; GNOME Foundation; Linux Foundation) | THE Linux desktop standard runtime. | None planned. |
| **OpenAPI 3.2.0** | KEEP (Linux Foundation OpenAPI Initiative) | THE API contract standard across ecosystems. | None planned — this is the cross-ecosystem unifier. |
| **Style Dictionary** | KEEP (Apache 2.0; Amazon-published) | THE design-token compilation standard. | None planned. |
| **svelte-flow** (not currently selected; would be used for Phase 1 node-editor canvas if needed) | **Vendor-replaceable — Phase 2 in-house** | If oyatie ever adopts svelte-flow for the Workflow Studio canvas, classify it as vendor-replaceable. The xyflow team (React Flow / Svelte Flow) ships under MIT, but the canvas surface is high-impact UX where oyatie wants control over rendering perf, accessibility, and Cedar-policy-aware node masking. | If adopted in Phase 1: Phase 2 trigger = Leptos migration (the canvas becomes Rust-native anyway as part of the Leptos shift). The Leptos canvas would be built on `leptos` + native SVG/Canvas, no JS canvas library wrap. |

**Why Leptos Phase 2 IS oyatie's in-house web shift**: the Rust-primary endgame collapses the web tier into the same workspace as the rest of oyatie. The Leptos build consumes `oya-client-shared-rust`, which the Linux GTK client also consumes. The Apple shared layer is Swift (platform-mandatory). The Android shared layer is Kotlin (platform-mandatory). The Windows shared layer is .NET (platform-mandatory). The ONE language ecosystem oyatie controls — Rust — owns web (Phase 2) + Linux desktop + every backend µservice. That IS the AWS/Google/Microsoft/Oracle pattern: the language ecosystem oyatie controls is what oyatie's in-house engineering doubles down on.

### Operational

1. Per-client `client-manifest.json` schema enforced by `oya-check-client-stack-discipline` fitness gate (per ADR-0186 wiring).
2. CI runs platform-specific smoke tests per surface:
   - Web SvelteKit: `pnpm test`, `playwright e2e`.
   - Web Leptos: `cargo test`, `wasm-pack test`.
   - Apple: `xcodebuild test` for each target.
   - Android: `./gradlew connectedAndroidTest`.
   - Windows: `dotnet test`.
   - Linux: `cargo test` + `flatpak-builder` build smoke.
3. Style Dictionary token pipeline runs per-PR; drift between `clients/design-tokens/*.json` and per-platform compiled outputs fails CI.

## Rollback

Per-surface rollback is `git revert` of the surface's `clients/<surface>/` changes. The OpenAPI contract is backward-compatible per ADR-0166 (schema registry). Other surfaces continue shipping during a single-surface rollback.

Phase 2 web migration rollback: each surface migrates independently; partial rollback is per-surface flip back to SvelteKit if a Leptos migration regresses.

## References

- ADR-0064 — canonical-base + localization packs (design tokens compile per-pack overlays).
- ADR-0131 — per-microservice flat layout (client trees live under `microservices/<ms>/clients/`).
- ADR-0145 — inter-microservice communication reform (clients call the public OpenAPI REST gateway; gRPC stays internal).
- ADR-0148 — service-mesh canonical (clients enter the mesh via Envoy Gateway per ADR-0182).
- ADR-0182 — API gateway (north-south).
- ADR-0186 — observability backplane (clients ship OpenTelemetry trace context via OTLP).
- SvelteKit — https://svelte.dev/ ; current 2.55.0 (April 2026).
- Svelte 5 + runes — runes-first reactive primitives (stable).
- Vite — https://vite.dev/ ; current 8.0.13 (Rolldown-based; May 2026).
- TypeScript — https://www.typescriptlang.org/ ; current stable 6.0.3 (April 2026); 7.0 beta (Go-rewrite) available.
- Leptos — https://leptos.dev/ ; current 0.8.x; 1.0 outlook 2026.
- Swift — current 6.3 (Apple).
- Swift OpenAPI Generator — https://github.com/apple/swift-openapi-generator ; current 1.11.1 (April 2026); supports OpenAPI 3.0/3.1 + preliminary 3.2.
- Kotlin — current 2.3.0.
- Jetpack Compose — Android target current; Compose Multiplatform 1.11.0 (rejected for iOS per this ADR).
- KMP — Kotlin Multiplatform; stable for iOS since 2024 (rejected here for iOS-share; Android-scope-only).
- WinUI 3 — Windows App SDK 1.8 (released Sept 2025; serviced through March 2026; SDK 2.0 preview targeting .NET 10).
- .NET 10 — LTS released Nov 11-13, 2025; 3-year support.
- Microsoft Kiota — https://github.com/microsoft/kiota ; current 1.31.1 (April 2026).
- GTK 4 / gtk-rs — https://gtk-rs.org/ ; gtk4 crate current 0.11.3 (April 2026); Rust MSRV 1.83.
- libadwaita — https://gitlab.gnome.org/GNOME/libadwaita ; current 1.8 (Sept 2025; GNOME 49 series).
- Flatpak runtime — `org.gnome.Platform` GNOME 49/50 (GNOME 48 EOL 2026-03).
- Style Dictionary — token compilation pipeline.
- OpenAPI 3.2.0 — https://www.openapis.org/
- LTS-rotation cadence: versions current as of 2026-05-18; review per ADR-0098 (LTS pin policy).
