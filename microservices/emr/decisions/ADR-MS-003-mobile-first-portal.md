---
id: ADR-EMR-MS-003
status: Accepted
deciders: axis-emr, council-product, council-clinical, council-security
date: 2026-05-21
microservice: emr
purpose: Adopt a mobile-first design for the patient + caregiver portal, with web as a mobile-responsive companion rather than the primary surface.
related:
  - ADR-EMR-MS-001
  - ADR-EMR-MS-002
  - ADR-0188 passkey/WebAuthn as canonical auth
  - ADR-0244 tenant scoping
---

# ADR-EMR-MS-003: Mobile-first patient + caregiver portal

## Status

Accepted — 2026-05-21.

## Context

The dominant US patient portal is Epic MyChart. MyChart was originally a web product (2003) that gained a mobile app circa 2010; the web portal remains feature-equivalent and is the primary surface for many patients. Cerner HealtheLife, athenaCommunicator, NextGen Patient Portal, and Allscripts FollowMyHealth follow the same pattern: web-first with mobile-as-companion.

The empirical reality in 2026 is inverse to this design heritage:

- Per Kaiser Family Foundation (2024): 78% of US adults aged 18–64 access their patient portal primarily via mobile app; 89% of those under 35 do.
- Per Pew Research (2024): 92% of all healthcare-portal sessions are on mobile when the patient has both web + mobile available.
- Margaret Chen (PRD §B persona 4) does her chart review on her phone at the kitchen table or during her commute.
- Caregiver-proxies (James Thompson, persona 5) overwhelmingly use mobile.

Most competitor portals are mobile-responsive web first; native-app capabilities (push notifications, biometric login, offline-resilient drafts, camera-based document capture, Apple Health / Google Health integration) are second-class.

oyatie's strategic position is mobile-first: design the patient + caregiver portal experience to be best-in-class on mobile native, with web as a mobile-responsive web companion (not a separate-experience desktop product).

## Decision

1. **Mobile native is the primary patient-portal surface.** iOS app (Swift / SwiftUI) + Android app (Kotlin / Jetpack Compose) are built first; UX design dominates over web.
2. **Web is a mobile-responsive companion.** A single web surface (React, with WebKit-first design) renders well on mobile-browser + desktop-browser. Desktop receives no extra feature surface beyond what mobile-responsive supports.
3. **Authentication is passkey-first** (per ADR-0188). Native apps use platform passkeys (iOS Keychain / Android Credential Manager) + biometric. Web uses WebAuthn passkeys + browser-side biometric where available.
4. **Push notifications are first-class.** APNs on iOS, FCM on Android. Tenant-pack overlay can disable on regulated cells. Patient receives push for: result available, message from care team, appointment reminder, refill ready, education content assigned, billing statement.
5. **Apple Health + Google Health integration is a Wave-2 follow-up.** Per ADR-MS-001 the `vital` BC supports patient-contributed vitals; the mobile-app integration with the platform health APIs ships after MVP.
6. **Native apps offline-resilient.** Patients on subway / rural-connectivity must browse cached chart data (last fetched) + queue messages to send when reconnected. Sync via the FHIR R5 SubscriptionTopic.
7. **Document capture via camera.** Patients can capture insurance card, government ID (for proxy verification), prior-encounter records (e.g., a paper note from an out-of-network specialist) and attach via the secure attachment pipeline.

## Native app stack

- **iOS:** Swift + SwiftUI, deployment target iOS 18+. Per global memory `feedback_rust_strict_only_no_python_2026_05_20`, Swift is an authorized frontend-only language. Build via Xcode + xcodebuild; CI lane on macOS-26-Apple-Silicon-M5+ runners.
- **Android:** Kotlin + Jetpack Compose, minSdkVersion 31 (Android 12). Per global memory, Kotlin is an authorized frontend-only language. Build via Gradle Kotlin DSL; CI lane Ubuntu 24.04.
- **Backend bridge:** FHIR R5 REST + WebSocket subscriptions via the same `oya-emr-rest` µservice.

## Web stack

- React (TypeScript). Build via Vite. WebKit-first responsive design. Web is canonical accessibility surface (WCAG 2.2 AA + WCAG 2.2 AAA where feasible).

## Rejected alternatives

- **Web-first, mobile-as-companion (Epic MyChart pattern).** Rejected — does not match empirical patient usage in 2026.
- **Hybrid (React Native).** Rejected — React Native sacrifices native passkey integration depth + native biometric UX + native offline-storage performance. Healthcare-portal UX bar is too high to surrender these.
- **PWA-only.** Rejected — PWA push-notification + biometric story remains weaker than native on iOS (Safari constraints).
- **Single iOS app first, defer Android.** Rejected — Android dominates KR + EU + most APAC; mandatory parity.

## Consequences

### Positive

- Best-in-class patient experience captures the "Margaret would prefer this over MyChart" market.
- Per-platform features (passkey, biometric, push, offline, camera, Health Connect) materially differentiate oyatie from competitors.
- Caregiver-proxy workflow (US-PORT-013) more friendly on mobile (Face ID / biometric proxy-switch).

### Negative

- Two native platforms + web = three frontend codebases. CI footprint expands.
- iOS and Android UX designers required as discipline-specific staff or vendor partners.
- App Store / Play Store review processes for healthcare apps include additional disclosure burden.

### Operational

- iOS app deployable via TestFlight beta → App Store; Android via internal track → Play Store; web via per-tenant subdomain.
- Per-tenant white-label app supported via App Store Provider Certificate + Android signing key per tenant (B2B-IDN tenants ship branded app under their own brand).
- Mobile-app crash-reporting via tenant-scoped aggregation; PHI scrubbed at crash-report-emit.

## Verification

- App Store rejection rate < 5% across releases.
- Mobile-app MAU > 35% of active patient panel per US-PORT-011.
- Native-passkey-login success rate > 99% per tenant cell.

## References

- ADR-0188 passkey/WebAuthn as canonical auth.
- ADR-0244 tenant scoping.
- Memory `feedback_rust_strict_only_no_python_2026_05_20` — Swift/Kotlin authorized for frontend.
- Pew Research 2024 portal-usage survey.
- KFF 2024 patient-portal survey.
- Epic MyChart product history (KLAS).
- Apple Health + Google Health API references.
