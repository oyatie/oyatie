---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-identity
microservice: identity
status: Proposed
sales_segment: shared-substrate
tier: hero-substrate
tier_subtype: substrate-identity
tier_classification_rationale: |
  The identity µservice is the universal authentication + authorization
  substrate on which every other product surface depends. Per ADR-0242 oyatie
  itself is a tenant of its own platform; per ADR-0244 every authenticated
  principal carries tenant context as a first-class field; per ADR-0243 every
  authorization check is a Cedar evaluation against the principal. Identity
  is the precondition for any tenant-scoped read or write, so it is
  hero-substrate (not hero-product) — its consumers are every other µservice
  + every product surface (B2C personal, B2B work, oyatie-internal).
keystone-bundle: 2026-05-20-foundational-doctrine
milestone_first_ship: M01-foundation
related_adrs:
  - ADR-0028
  - ADR-0056
  - ADR-0105
  - ADR-0117
  - ADR-0131
  - ADR-0140
  - ADR-0145
  - ADR-0148
  - ADR-0156
  - ADR-0157
  - ADR-0162
  - ADR-0173
  - ADR-0175
  - ADR-0179
  - ADR-0182
  - ADR-0183
  - ADR-0187
  - ADR-0188
  - ADR-0189
  - ADR-0190
  - ADR-0191
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0255
  - ADR-0292
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
related_specs:
  - /specs/microservices/identity.json
  - /specs/tenant-model.json
  - /specs/per-microservice-flat-layout.json
  - /specs/step-up-acr-classes.json
related_memories:
  - oyatie-is-a-tenant
  - tenant-as-universal-scoping-primitive
  - cedar-as-universal-gate
  - byok-everywhere-credentials
  - mls-rfc-9420-e2ee-personal-messenger
  - compliance-pack-primitive
  - quality-performance-scalability-bar
date: 2026-05-20
owner_team: axis-identity + council-privacy + ops-security
doc_status: published
tenant_scoped: true
audience_modes:
  - B2C-personal
  - B2B-work
  - oyatie-internal-tenant
benchmarks:
  - okta
  - auth0
  - microsoft-entra-id
  - google-workspace-identity
  - apple-sign-in
  - kakaotalk-login
  - line-login
  - wechat-oauth
  - aws-cognito
  - aws-iam-identity-center
  - duo-security
  - yubico-fido2
  - zitadel
  - keycloak
  - one-login
  - ping-identity
---

# PRD-identity: Universal Identity + Authentication + Authorization Substrate

> Hero substrate. Powers every authenticated surface oyatie ships — B2C personal (passwordless passkey-first signup, social IdP federation, multi-device key sync), B2B work (SAML / OIDC SSO, SCIM provisioning, MFA enforcement, step-up auth), oyatie-internal tenant (per ADR-0242 oyatie runs on its own platform; no special code paths). Per ADR-0243 every authz check is a Cedar evaluation; per ADR-0244 every principal carries tenant context; per ADR-0255 §D-4 provider_credential_mode extends to tenant-owned IdP credentials (B2B SAML / OIDC).

---

## A. Problem

### A.1 Why identity needs its own µservice

Every authenticated user interaction starts in identity. If identity fails or leaks, every other µservice's tenant isolation, audit trail, Cedar gate, and compliance pack falls. The problems an identity substrate solves are:

1. **Single OIDC issuer + JWKS** for the fleet, so Cedar policies anywhere can validate `principal.iss` against a known set.
2. **Per-pack residency** for identity (KR data stays in KR; EU data stays in EU per ADR-0179) without cross-pack federation.
3. **B2B SSO** (SAML 2.0, OIDC, OAuth 2.1) so tenants can BYO IdP (Okta, Microsoft Entra, Google Workspace).
4. **B2C consumer login** (Apple Sign-In, Google Sign-In, KakaoTalk Login, LINE Login, WeChat OAuth, Naver Login, email-password, phone-OTP-where-allowed, passkey-first).
5. **SCIM 2.0 provisioning** for B2B lifecycle (hire, promote, terminate).
6. **MFA** with phishing-resistant default (FIDO2 / WebAuthn passkeys); SMS-OTP only where regulator-required (KR financial).
7. **Step-up auth** so sensitive ops require a fresh, higher-assurance factor (per `docs/standards/step-up-auth-classes.md`).
8. **Minor handling** per ADR-0292: COPPA <13 refusal, KOSA 14-17 tier, EU age verification per Member State, KR youth protection.
9. **Audit + provenance** for every login, MFA, step-up, SCIM event, IdP federation event.

### A.2 What competitors get wrong

Hyperscaler precedents:

- **Okta** (industry SSO leader) — strong B2B, weak B2C consumer flows; expensive at scale.
- **Auth0** (developer-first IdP) — flexible but easy to misconfigure (the Atlassian breach 2024 root-cause was misconfigured Auth0 callback URLs).
- **Microsoft Entra ID** (formerly Azure AD) — deeply integrated with M365; per-tenant cost; KR/JP/CN localisation gaps.
- **Google Workspace Identity** — strong consumer-grade, weaker enterprise lifecycle.
- **AWS Cognito** — flexible but baroque; per-user pricing; UX flows require self-build.
- **Apple Sign-In** — privacy-first consumer; B2C-only; no SCIM or SAML.
- **Zitadel** (open-source) — used as the canonical identity backend (per ADR-0187); good multi-tenant model.
- **Keycloak** — open-source; flexible but operationally heavy at scale.
- **KakaoTalk Login / LINE Login / WeChat OAuth / Naver Login** — region-dominant consumer IdPs; integration MUST be first-class for KR/JP/CN/TW markets.

Failure modes observed:

1. **Lapsus$ Okta breach 2022** — a third-party-support-engineer's session was stolen. Fix: principal-of-least-privilege for support roles + session-binding to device-bound credentials.
2. **Twitter SMS-2FA bypass 2023** — SMS-OTP was bypassed via SIM-swap. Fix: passkey-default; SMS only where regulator-required.
3. **MoveIT identity leak 2023** — admin credentials in a customer-facing tool. Fix: per-µservice service accounts via short-lived tokens; no long-lived admin keys.
4. **Naver Login privacy debate 2024** — Naver passed over-broad consumer profile to merchants. Fix: minimal-scope OIDC + Cedar gate on attribute disclosure.

### A.3 What "good" looks like

A B2C consumer signs in with one tap (Apple/Google/KakaoTalk passkey-conditional-UI). They get an OIDC ID token + access token + refresh token; ID token claims include `tenant_id=__personal__/<user>`, `acr=passkey`, `purpose=`, `data_class=`, `age_class=`, `jurisdiction_code=`. Every downstream µservice validates the token via JWKS, extracts tenant context, runs Cedar.

A B2B enterprise admin federates Okta as an upstream IdP; users SSO in once; SCIM pushes user lifecycle; sensitive operations (key rotation, tenant deletion, billing change) prompt for step-up to `acr=critical` (hardware key + 4-eye approval). Every event audit-chain sealed.

oyatie's own employees sign in to internal tools the exact same way (per ADR-0242). No special code paths.

---

## B. Target Users (Personas)

### B.1 B2C personas

#### Persona B2C-1 — "Ji-min, Korean university student signing up for personal messenger"
- **Goals**: sign up with KakaoTalk Login (her primary identity), set up FaceID passkey, multi-device sync to her Galaxy Tab, recover access if she loses her phone.
- **Frustrations**: foreign apps that don't accept KakaoTalk Login; SMS-OTP that fails on her travel SIM; password fields that require character classes; no account recovery without identity verification.
- **Tech comfort**: high (multi-app pay/social stack; passkey-savvy).
- **Locale + device**: ko-KR, KST, iPhone 15 (primary), Galaxy Tab S9 (secondary), MacBook Air for school.

#### Persona B2C-2 — "Marcus, German indie creator on multiple devices"
- **Goals**: sign in with Google (no separate passwords), enable passkey on his iPhone + MacBook + Pixel, use a hardware YubiKey as backup, no surveillance-capitalism profile-merging across his accounts.
- **Frustrations**: opaque "share with this app" consent screens; per-device passkey re-enrollment; lost-key recovery requiring 24h delay.
- **Tech comfort**: high (developer-adjacent creator; understands passkey + WebAuthn).
- **Locale + device**: de-DE, CEST, MacBook Pro M3 + iPhone 15 Pro + Pixel 8 + YubiKey 5C NFC.

#### Persona B2C-3 — "Sofia, Brazilian retiree, low-tech-comfort"
- **Goals**: sign in with her phone number + SMS, doesn't want a password; doesn't understand "passkey"; just wants to use the messenger to talk to her grandchildren.
- **Frustrations**: complicated setup; English error messages; biometric-only login when she doesn't trust facial recognition.
- **Tech comfort**: low (calls grandchildren when stuck; uses larger fonts).
- **Locale + device**: pt-BR, BRT, Android mid-range (Samsung A24), reading-glasses-friendly UI required.

#### Persona B2C-4 — "Alex (13) and parent guardian, COPPA-refusal case"
- **Goals**: Alex wants to sign up; system MUST refuse <13 OR route through parental-consent flow per local law; parent receives notification; minor data handling per ADR-0292.
- **Frustrations**: silent age-gating; over-collection of minor data; lack of parent oversight visibility.
- **Tech comfort**: Alex high, parent medium.
- **Locale + device**: en-US, family iPad.

#### Persona B2C-5 — "Yuki, Japanese teen (16), KOSA tier"
- **Goals**: sign up via LINE Login; parents notified per ADR-0292 KOSA tier; minor-PII handling per regulator; can use the messenger with privacy controls defaulted to "private".
- **Frustrations**: feature-restrictions that feel patronizing; difficulty upgrading to adult account on 18th birthday.
- **Tech comfort**: high (digital-native).
- **Locale + device**: ja-JP, JST, iPhone SE3 (school-allowed).

### B.2 B2B personas

#### Persona B2B-1 — "IT-Admin Ingrid, Berlin SaaS company with 500 employees"
- **Goals**: federate Microsoft Entra ID as upstream IdP; SCIM-push users + groups; enforce MFA via Entra Conditional Access (oyatie respects upstream MFA assertion); set role mappings (`group: engineering → role: viewer`); step-up auth required for tenant-admin ops.
- **Frustrations**: SCIM provisioning that creates duplicates; group nesting; deprovisioning lag (terminated employee retains access).
- **Tech comfort**: very high (enterprise IT veteran; SAML / SCIM / SSO certified).
- **Locale + device**: de-DE + en-US (cross-language admin console), CEST, Windows desktop primary.

#### Persona B2B-2 — "CISO Carl, US Fortune-500 enterprise"
- **Goals**: enforce passkey-only login for all employees (no passwords ever); audit every step-up auth grant; route 4-eye approval for prod ops; FedRAMP-style audit evidence; hardware-token (FIDO2 + PIV smart card) required for `acr=critical`.
- **Frustrations**: vendors that can't enforce passkey-only; opaque audit log gaps; long incident-response cycle on credential theft.
- **Tech comfort**: very high (security-architect background).
- **Locale + device**: en-US, ET, Windows + iPad Pro for board reviews.

#### Persona B2B-3 — "HR Manager Hyo, Korean enterprise wanting native SCIM-equivalent"
- **Goals**: push hire/promote/terminate from her HRIS (Korean SaaS: Saramin / Recruit) to oyatie; non-SCIM HRIS so requires a custom adapter; KR-PIPA-compliant data flow; KR employee names in Hangul, English, Hanja.
- **Frustrations**: foreign IdPs that don't speak KR HR conventions; manual CSV imports; lifecycle propagation delays.
- **Tech comfort**: medium-high.
- **Locale + device**: ko-KR, KST, Windows + KakaoTalk for team chat (not oyatie product but typical in KR enterprise).

### B.3 Internal persona

#### Persona INT-1 — "Security Engineer Sasha, oyatie ops-security team"
- **Goals**: monitor identity-misuse signals (impossible-travel, password-spray, SCIM-floods); rotate signing keys quarterly; respond to identity-related incidents; submit FIDO MDS3 metadata updates monthly.
- **Frustrations**: noisy alerts; manual incident-response; lack of forensic timeline reconstruction.
- **Tech comfort**: very high (CISSP / OSCP-grade).

---

## C. User Stories

Stories are identity-µservice specific. Do NOT duplicate `docs/user-stories/b2c-consumer-surfaces.md` or `b2b-work-surfaces.md`; reference them by ID; add identity-specific NEW stories.

### US-identity-01 — Passkey-first consumer signup
- **As** Ji-min (B2C-1)
- **I want** to tap "Sign up with KakaoTalk", confirm in KakaoTalk app, then enroll a FaceID passkey
- **so that** I never type a password and future signins are one-tap.
- **Acceptance criteria**:
  1. KakaoTalk OAuth flow initiates within 1s of tap; KakaoTalk app push received.
  2. After OAuth callback, oyatie issues OIDC ID token + creates personal-tenant `__personal__/<user_id>`.
  3. Passkey enrollment ceremony (WebAuthn L3 `navigator.credentials.create`) completes within 5s.
  4. Subsequent signin uses conditional-UI passkey (no username field needed); ID token issued within 200ms server-side.
- **Accessibility AC**: WCAG 2.2 AA on signup screens; screen-reader friendly; voice-over announcements during passkey ceremony.
- **i18n AC**: signup screens in ko-KR; KakaoTalk consent screen in ko-KR; passkey "FaceID" terminology localized.

### US-identity-02 — Multi-device passkey sync
- **As** Marcus (B2C-2)
- **I want** my passkey to sync between my iPhone, MacBook, and Pixel
- **so that** I can sign in from any of my devices.
- **Acceptance criteria**:
  1. iCloud Keychain syncs passkey across Apple devices automatically.
  2. Cross-platform: enroll a separate passkey on Pixel via Google Password Manager.
  3. Account "Devices" page lists all enrolled passkey credentials with last-used time + ability to revoke any.
  4. Lost-device flow: any other passkey OR YubiKey OR account-recovery flow can revoke the lost device.
- **Accessibility AC**: device list table accessible; revoke action keyboard-accessible.
- **i18n AC**: device names show in user locale; timestamps in user TZ.

### US-identity-03 — Phone-number signup for low-tech-comfort
- **As** Sofia (B2C-3)
- **I want** to sign up with my phone number + SMS-OTP
- **so that** I don't need to remember a password.
- **Acceptance criteria**:
  1. Phone number entered; SMS sent within 5s; OTP code valid for 5 minutes.
  2. After OTP confirmation, oyatie issues OIDC token + creates personal tenant.
  3. UI offers passkey enrollment AFTER successful signup but does NOT require it.
  4. Re-signin via phone+SMS available as fallback always.
  5. SMS-OTP fallback is REGION-GATED: only available in pack-br (BR), pack-id (ID), pack-vn (VN); pack-eu and pack-us refuse SMS-OTP for new signups per NIST SP 800-63-3 + EBA RTS.
- **Accessibility AC**: large-text mode; OTP input accepts paste; voice-over reads OTP.
- **i18n AC**: SMS body in pt-BR; UI in pt-BR; OTP screen with simple language.

### US-identity-04 — COPPA <13 refusal
- **As** Alex (B2C-4)
- **I want** the system to detect that I am <13 and refuse signup
- **so that** no minor account is created and my parent is informed.
- **Acceptance criteria**:
  1. During signup, if `birth_year` implies age <13 (US), Cedar `identity::age_assurance::coppa_refuse` returns forbid.
  2. UI presents a parental-portal link instead; no account is created.
  3. Audit-chain emits `IdentitySignupRefused{reason="coppa_age", subject_hashed_id, ip}`.
  4. Parent (if a contact email was provided) receives notification within 5 minutes.
- **Accessibility AC**: refusal screen empathetic, non-blame; parent-link clearly labeled.
- **i18n AC**: refusal copy in en-US for now; ADR-0292 expansion adds other Member-State locales.

### US-identity-05 — KOSA tier (14-17) signup
- **As** Yuki (B2C-5)
- **I want** to sign up as a minor with parental notification
- **so that** the platform enforces KOSA-tier defaults (privacy-default, parent visibility, time-limit option).
- **Acceptance criteria**:
  1. Birth year implies age 14-17 → Cedar `identity::age_assurance::kosa_minor_tier` returns context `{tier: 14_17, parental_notify: required, defaults: privacy_max}`.
  2. Parent email/phone collected with explicit consent; notification dispatched.
  3. Account opens with privacy defaults; opt-outs require parental-portal action.
  4. On 18th birthday: account state moves `kosa_minor → adult`; defaults remain user-controlled.
- **Accessibility AC**: forms accessibility AA; parental consent step explicit.
- **i18n AC**: ja-JP signup copy; parent notification in ja-JP.

### US-identity-06 — B2B SAML SSO with Okta upstream
- **As** Ingrid (B2B-1)
- **I want** to federate our Okta as an upstream IdP
- **so that** users SSO in without separate oyatie credentials.
- **Acceptance criteria**:
  1. Admin uploads Okta SAML metadata XML (or enters issuer URL).
  2. oyatie issues a SAML SP metadata XML back for Okta config.
  3. Test login: user redirected to Okta → authenticates → returns with SAML assertion.
  4. oyatie verifies assertion signature, extracts `nameID` + attributes, issues OIDC token with `tenant_id=<tenant>, idp=okta`.
- **Accessibility AC**: admin SAML config UI accessibility AA.
- **i18n AC**: admin console in de-DE + en-US.

### US-identity-07 — B2B OIDC federation with Microsoft Entra
- **As** Ingrid (B2B-1)
- **I want** to use OIDC federation instead of SAML for cleaner UX
- **so that** logins are faster.
- **Acceptance criteria**:
  1. Admin enters Microsoft Entra tenant ID + client ID; secret stored as SecretReference per ADR-0255.
  2. Test login: redirect → Entra auth → callback with OIDC `id_token`.
  3. oyatie validates Entra JWKS, extracts claims, issues oyatie OIDC token.
  4. Group claims mapped to oyatie roles via admin-configured mapping.
- **Accessibility AC**: admin config UI AA.
- **i18n AC**: per-locale.

### US-identity-08 — SCIM 2.0 user provisioning
- **As** Ingrid (B2B-1)
- **I want** Okta to push user lifecycle (hire, promote, terminate) via SCIM 2.0
- **so that** I don't maintain two user lists.
- **Acceptance criteria**:
  1. Admin generates a per-tenant SCIM bearer token (rotates every 90 days).
  2. Okta SCIM POST `/Users` creates a user; SCIM PUT updates; SCIM DELETE soft-deletes (sets `active=false`).
  3. SCIM `/Groups` syncs groups; group memberships drive role assignment via mapping.
  4. SCIM `/Schemas` advertises supported attributes per RFC 7643.
  5. SCIM `/ServiceProviderConfig` advertises capabilities.
- **Accessibility AC**: N/A (server-to-server).
- **i18n AC**: error messages localized when SCIM provisioning fails on tenant request.

### US-identity-09 — Non-SCIM HRIS integration
- **As** HR Manager Hyo (B2B-3)
- **I want** to push hire/terminate from our KR HRIS (non-SCIM)
- **so that** I don't manage two user lists.
- **Acceptance criteria**:
  1. Plug-in adapter contract per ADR-0187: `HrisAdapter` trait with `list_users`, `get_user_changes`, `apply_to_zitadel`.
  2. Adapter polls HRIS every 15 minutes; computes diff; applies to oyatie identity via internal API.
  3. Adapter audit-chain emits per-event.
  4. Initial adapters: Saramin, Recruit, BambooHR, Rippling, Workday, Personio.
- **Accessibility AC**: N/A.
- **i18n AC**: KR-HRIS adapter handles Hangul + Hanja + English name variants.

### US-identity-10 — Step-up to acr=critical for tenant deletion
- **As** Ingrid (B2B-1) deleting our oyatie tenant
- **I want** to be prompted for step-up authentication (passkey + IT-approval) before the delete proceeds
- **so that** a compromised session cannot accidentally destroy our data.
- **Acceptance criteria**:
  1. Tenant-delete endpoint refuses unless `principal.acr == "critical"`.
  2. Step-up flow: user prompted to re-authenticate with passkey + a second admin approves via 4-eye.
  3. Audit-chain emits `IdentityStepUpGranted{acr=critical, reason=tenant_delete, approvers=[a,b]}`.
  4. Step-up valid for 5 minutes; after that, re-step-up required.
- **Accessibility AC**: step-up UI accessibility AA; 4-eye approval UI clear.
- **i18n AC**: step-up prompts localized.

### US-identity-11 — Hardware-key as backup
- **As** Marcus (B2C-2)
- **I want** to enroll a YubiKey 5C NFC as a backup credential
- **so that** if I lose my phone, I still have access.
- **Acceptance criteria**:
  1. WebAuthn enrollment with `authenticatorAttachment=cross-platform`; YubiKey accepted.
  2. AAGUID validated against FIDO MDS3; YubiKey AAGUID allowed.
  3. Credential stored; UI shows it in Devices list with badge "Hardware key".
  4. Signin via YubiKey works on any oyatie web/desktop client.
- **Accessibility AC**: YubiKey enrollment UI accessibility AA.
- **i18n AC**: device label "YubiKey" localized.

### US-identity-12 — Account recovery via account-recovery-flow
- **As** Sofia (B2C-3) who lost her phone
- **I want** to recover access via email + ID-document verification
- **so that** I can sign in again.
- **Acceptance criteria**:
  1. Recovery flow accepts email + recovery code OR ID-document upload.
  2. 24h cooldown to prevent social engineering (KR-PIPA-compliant timeline).
  3. Recovery completion grants `acr=elevated` only; `critical` ops still require step-up after recovery.
  4. All recovery attempts audit-chain logged.
- **Accessibility AC**: recovery UI accessibility AA; supports large-text mode.
- **i18n AC**: recovery copy in pt-BR.

### US-identity-13 — Passkey-only enforcement for B2B tenant
- **As** Carl (B2B-2)
- **I want** to enforce passkey-only for our oyatie tenant (no passwords ever)
- **so that** phishing is structurally impossible.
- **Acceptance criteria**:
  1. Tenant setting `auth.password_allowed=false` + `auth.factor_required=[webauthn]`.
  2. Users without passkey are forced to enroll on next login.
  3. Admins can grant a one-time exception (audit-chain logged).
- **Accessibility AC**: enforcement UI clear; user-side enrollment accessible.
- **i18n AC**: enforcement copy localized.

### US-identity-14 — JIT IT-approval for sensitive ops
- **As** Carl (B2B-2)
- **I want** prod-ops (e.g., key rotation) to require a second IT-team approver
- **so that** single-actor compromise cannot escalate.
- **Acceptance criteria**:
  1. Cedar policy `identity::step_up::critical` requires `principal.approval_count >= 2`.
  2. Initial action queued; admin notified to approve; approver re-authenticates.
  3. Approval granted within 30 minutes (configurable) OR action fails closed.
- **Accessibility AC**: approval notification accessible.
- **i18n AC**: per-locale.

### US-identity-15 — Conditional-UI passkey signin
- **As** Marcus (B2C-2)
- **I want** the browser to show my passkey as a suggestion when I focus the username field
- **so that** I tap once and signin.
- **Acceptance criteria**:
  1. WebAuthn conditional UI enabled on signin page.
  2. Browser surfaces saved passkeys for the origin.
  3. Selecting one completes the assertion ceremony in 1-2 taps.
- **Accessibility AC**: focus-management correct; screen-reader announces passkey option.
- **i18n AC**: passkey UI follows browser locale.

### US-identity-16 — Session binding to device
- **As** Carl (B2B-2)
- **I want** sessions to be device-bound so a stolen cookie can't replay
- **so that** the Lapsus$-style support-engineer-cookie-theft attack is mitigated.
- **Acceptance criteria**:
  1. OAuth DPoP (Demonstration of Proof of Possession) RFC 9449 implemented.
  2. Each request carries `DPoP` header signed by a per-session key.
  3. Stolen access token without the corresponding private key is unusable.
  4. Per ADR-0157 api-gateway validates DPoP signature.
- **Accessibility AC**: N/A (transparent to user).
- **i18n AC**: error messages on DPoP failure localized.

### US-identity-17 — OAuth device-code flow for CLI
- **As** a developer using the `oya` CLI on a server with no browser
- **I want** to authenticate via device-code flow
- **so that** I get a token without pasting secrets.
- **Acceptance criteria**:
  1. CLI requests device code via `POST /oauth/device_authorization`.
  2. CLI prints URL + user-code; user opens URL in browser, enters code, completes signin.
  3. CLI polls and receives token within 30s of user confirmation.
  4. Token scope limited to operations declared at device-code request.
- **Accessibility AC**: device-code printed in large legible font; copyable.
- **i18n AC**: per-locale (CLI inherits user locale env).

### US-identity-18 — Apple Sign-In as B2C IdP
- **As** Marcus (B2C-2)
- **I want** to sign in with Apple ID
- **so that** my Apple privacy preferences (relay email, no profile-merging) are honored.
- **Acceptance criteria**:
  1. Apple Sign-In implemented per Apple specs; relay-email accepted.
  2. oyatie does NOT request profile fields beyond `sub`, `email` (relay-email is fine).
  3. Sign-in completes; oyatie OIDC token issued.
- **Accessibility AC**: Apple-provided button used (accessibility-validated by Apple).
- **i18n AC**: Apple Sign-In follows browser locale.

### US-identity-19 — Google One-Tap consumer signin
- **As** Ji-min (B2C-1) on web
- **I want** Google One-Tap signin to appear if I'm signed into Google
- **so that** signin is frictionless.
- **Acceptance criteria**:
  1. Google One-Tap initialized on home page.
  2. User taps; OIDC token from Google validated; oyatie token issued.
  3. If user prefers KakaoTalk, One-Tap is dismissable.
- **Accessibility AC**: One-Tap prompt accessible; dismissable via keyboard.
- **i18n AC**: One-Tap follows browser locale.

### US-identity-20 — KakaoTalk Login B2C
- **As** Ji-min (B2C-1)
- **I want** to sign in with KakaoTalk
- **so that** I use my existing KR identity.
- **Acceptance criteria**:
  1. KakaoTalk OAuth 2.0 flow implemented.
  2. After auth, oyatie issues token with minimal scope (`sub`, `email` optional, no profile merge).
  3. UI clearly shows what data oyatie receives from KakaoTalk.
- **Accessibility AC**: consent UI accessibility AA.
- **i18n AC**: KR consent copy.

### US-identity-21 — LINE Login B2C
- **As** Yuki (B2C-5)
- **I want** to sign in with LINE (her primary JP identity)
- **so that** signup is one tap.
- **Acceptance criteria**:
  1. LINE Login implemented per LINE specs.
  2. Minimal scope (`profile`, `openid`).
  3. KOSA-tier age inferred from LINE profile birthday if provided; else explicit form.
- **Accessibility AC**: per-locale.
- **i18n AC**: ja-JP.

### US-identity-22 — WeChat OAuth B2C (zh-Hans cross-border)
- **As** a Chinese-speaking consumer outside CN (pack-sg / pack-au)
- **I want** to sign in with WeChat
- **so that** I use my existing identity.
- **Acceptance criteria**:
  1. WeChat OAuth 2.0 implemented; only available where pack permits (not CN-on-CN at M01 due to data-residency complexity).
  2. Token issued; minimal scope.
- **Accessibility AC**: per-locale.
- **i18n AC**: zh-Hans.

### US-identity-23 — Naver Login B2C
- **As** Ji-min (B2C-1) who prefers Naver
- **I want** to sign in with Naver (sibling KR IdP to Kakao)
- **so that** I have choice.
- **Acceptance criteria**:
  1. Naver OAuth 2.0 implemented.
  2. Minimal scope.
- **Accessibility AC**: per-locale.
- **i18n AC**: ko-KR.

### US-identity-24 — Token introspection for µservices
- **As** a downstream µservice author
- **I want** to validate an incoming Bearer token
- **so that** I can extract tenant + principal claims.
- **Acceptance criteria**:
  1. `GET /oauth/jwks` returns current signing keys; cached for 24h.
  2. Per ADR-0189 + ADR-0244, ID token claims include `tenant_id`, `acr`, `purpose`, `data_class`, `age_class`, `jurisdiction_code`.
  3. Stateless verification (no per-token network call typically).
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-identity-25 — Token revocation
- **As** Sasha (INT-1) responding to a credential incident
- **I want** to revoke tokens for a specific user immediately
- **so that** an exfiltrated token stops working.
- **Acceptance criteria**:
  1. `POST /oauth/revoke` per RFC 7009 with token; server enters JWT `jti` in deny-list.
  2. Per ADR-0157 api-gateway checks deny-list at edge; reject if hit.
  3. Deny-list TTL = token expiry; auto-clean.
- **Accessibility AC**: revoke UI accessibility AA.
- **i18n AC**: per-locale.

### US-identity-26 — Impossible-travel detection
- **As** Sasha (INT-1)
- **I want** alerts when a user signs in from two distant geographies within an impossible window
- **so that** account-takeovers surface early.
- **Acceptance criteria**:
  1. Sign-in events emit IP + geo lookup; risk engine computes "is travel feasible at jet speed".
  2. Score > threshold → step-up required + alert.
  3. False-positive feedback loop reduces noise.
- **Accessibility AC**: alert UI accessibility AA.
- **i18n AC**: ops alerts en-US.

### US-identity-27 — Password-spray attack detection
- **As** Sasha (INT-1)
- **I want** to detect distributed password-spray (one password across many accounts)
- **so that** automated attacks are blocked.
- **Acceptance criteria**:
  1. Failed-signin attempts indexed by `(password_hash_prefix, source_ip_block)`.
  2. > N failures from same IP block triggers block.
  3. Per-account lockout after 5 failures in 5 minutes; 15-minute cool-off.
- **Accessibility AC**: locked-out user gets clear unlock-flow.
- **i18n AC**: locale.

### US-identity-28 — Refresh-token rotation
- **As** a long-lived mobile app
- **I want** refresh tokens to rotate on every use
- **so that** stolen refresh tokens have a short useful lifespan.
- **Acceptance criteria**:
  1. Each refresh exchange returns a new refresh token; old one invalidated.
  2. Reuse of old refresh detected → entire session family invalidated; user re-auth.
- **Accessibility AC**: N/A.
- **i18n AC**: re-auth prompt localized.

### US-identity-29 — Per-pack residency
- **As** Ji-min (B2C-1) in KR
- **I want** my identity data to stay in pack-kr
- **so that** PIPA + sovereignty is honored.
- **Acceptance criteria**:
  1. Per ADR-0179, pack-kr identity data lives in OCI ap-seoul-1; no cross-pack federation.
  2. JWKS issuer `https://identity-kr.oyatie.com` is pack-specific.
  3. Cross-pack reads refused at Cedar gate.
- **Accessibility AC**: N/A.
- **i18n AC**: residency disclosure in ko-KR.

### US-identity-30 — JWKS rotation
- **As** Sasha (INT-1)
- **I want** to rotate signing keys quarterly (and emergency-rotate on incident)
- **so that** compromised keys have bounded impact.
- **Acceptance criteria**:
  1. JWKS endpoint exposes BOTH old + new keys during overlap window (1 week).
  2. After overlap, old key removed; tokens signed with old key expire naturally.
  3. Emergency-rotate flow available; 4-eye-approved.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-identity-31 — MFA enrollment ceremony accessibility
- **As** Sofia (B2C-3)
- **I want** the MFA enrollment ceremony to be screen-reader friendly
- **so that** I (low-vision user) can complete enrollment without help.
- **Acceptance criteria**:
  1. All steps have ARIA labels.
  2. Audio cue on success/fail.
  3. Large-text mode supported.
- **Accessibility AC**: WCAG 2.2 AA.
- **i18n AC**: pt-BR.

### US-identity-32 — Group + role mapping for B2B
- **As** Ingrid (B2B-1)
- **I want** to map upstream IdP groups to oyatie roles
- **so that** role assignment is automatic.
- **Acceptance criteria**:
  1. Admin UI: paste IdP group → select oyatie role.
  2. On user signin, group claim from IdP resolves to oyatie role.
  3. Audit-chain emits `IdentityRoleAssigned{user, group, role, mapped_at}`.
- **Accessibility AC**: mapping UI accessibility AA.
- **i18n AC**: de-DE + en-US.

### US-identity-33 — Deprovisioning on terminate
- **As** Ingrid (B2B-1)
- **I want** terminated employees to lose access within 5 minutes
- **so that** offboarding is timely.
- **Acceptance criteria**:
  1. SCIM DELETE `/Users/{id}` triggers soft-delete + token revocation.
  2. Per ADR-0157, api-gateway honors revocation within 5 minutes (next JWKS refresh).
  3. Audit-chain emit `IdentityUserDeprovisioned`.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-identity-34 — Suspicious-MFA-fatigue protection
- **As** Carl (B2B-2)
- **I want** to prevent MFA push-fatigue attacks (attacker spams pushes hoping user taps approve)
- **so that** my employees aren't tricked.
- **Acceptance criteria**:
  1. Push MFA requires number-matching (user types digits shown on the originating device).
  2. Failed match logs an alert.
  3. Per-user throttle on push count per minute.
- **Accessibility AC**: number-matching UI accessibility AA.
- **i18n AC**: per-locale.

### US-identity-35 — Audit log for compliance
- **As** Sasha (INT-1)
- **I want** every identity event (signin, MFA, SCIM, step-up) audit-chained
- **so that** SOC 2 / ISO 27001 audits have evidence.
- **Acceptance criteria**:
  1. Every event emits `AuditEvent` per ADR-0028.
  2. Per-pack retention per ADR-0162.
  3. Searchable via internal ops console.
- **Accessibility AC**: ops console accessibility AA.
- **i18n AC**: en-US (ops).

### US-identity-36 — DSAR for identity data
- **As** a EU consumer
- **I want** to download my identity data
- **so that** I can review what oyatie holds.
- **Acceptance criteria**:
  1. `POST /v1/dsar/identity/export` returns ZIP with profile, sign-in history, MFA enrollments, IdP federations.
  2. ZIP is signed (Ed25519); recipient can verify integrity.
  3. Export ready within 30 days of request.
- **Accessibility AC**: export UI accessibility.
- **i18n AC**: locale.

### US-identity-37 — Right-to-erasure of identity data
- **As** a EU consumer
- **I want** to delete my identity data
- **so that** I am forgotten.
- **Acceptance criteria**:
  1. DSAR-erasure removes profile fields; audit-chain retains event-meta but not personal identifiers.
  2. Per ADR-0156 PII registry, every field tagged; tombstoning applies per tag.
  3. Where retention required (legal/financial), subject link broken via key rotation per ADR-0255.
- **Accessibility AC**: erasure UI accessibility.
- **i18n AC**: locale.

### US-identity-38 — Multi-account on one device
- **As** Marcus (B2C-2)
- **I want** to have personal + work accounts on the same device
- **so that** I switch without re-auth.
- **Acceptance criteria**:
  1. Account-switcher in app UI lists all signed-in accounts.
  2. Each account has its own session + ID token.
  3. Switching does not leak context between accounts.
- **Accessibility AC**: switcher accessibility AA.
- **i18n AC**: per-locale.

### US-identity-39 — Cross-references to story banks
- See `docs/user-stories/b2c-consumer-surfaces.md#US-B2C-AUTH-*` for general consumer auth flows.
- See `docs/user-stories/b2b-work-surfaces.md#US-B2B-SSO-*` for enterprise SSO flows.
- This PRD's stories are identity-µservice-specific and add NEW scenarios beyond the story bank.

### US-identity-40 — Minor account upgrade on 18th birthday
- **As** Yuki (B2C-5) on her 18th birthday
- **I want** the system to graduate her from KOSA tier to adult
- **so that** she has full feature access.
- **Acceptance criteria**:
  1. On birthday-date, scheduled job moves state `kosa_minor → adult`.
  2. UI notifies; user can review settings.
- **Accessibility AC**: notification accessible.
- **i18n AC**: ja-JP.

### US-identity-41 — Tenant-admin role separation of duties
- **As** Carl (B2B-2)
- **I want** to enforce that one admin cannot grant themselves higher privileges
- **so that** SOX-style segregation-of-duties holds.
- **Acceptance criteria**:
  1. Cedar policy refuses self-grant of higher-tier role.
  2. Grant requires second admin's confirmation.
  3. Audit-chain logs both.
- **Accessibility AC**: confirmation UI accessibility.
- **i18n AC**: locale.

### US-identity-42 — Token scope downscoping
- **As** an OAuth client
- **I want** to request a token with reduced scope
- **so that** I follow least-privilege.
- **Acceptance criteria**:
  1. RFC 8693 token-exchange supported.
  2. Downscoped token is non-upgrade-able.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

---

## D. Functional Requirements

### D.1 OIDC issuer surface

| ID | Requirement |
|---|---|
| FR-I-01 | `GET /.well-known/openid-configuration` returns OIDC discovery doc per RFC 8414. |
| FR-I-02 | `GET /oauth/jwks` returns current signing keys (RSA + ECDSA + Ed25519 as needed). |
| FR-I-03 | `POST /oauth/token` issues access + ID + refresh tokens per OAuth 2.1. |
| FR-I-04 | ID token MUST include claims: `sub, iss, aud, exp, iat, nbf, tenant_id, acr, purpose, data_class, age_class, jurisdiction_code`. |
| FR-I-05 | Tokens signed with current signing key (rotated quarterly). |

### D.2 WebAuthn surface

| ID | Requirement |
|---|---|
| FR-I-10 | `POST /webauthn/register/begin` returns PublicKeyCredentialCreationOptions per L3 spec. |
| FR-I-11 | `POST /webauthn/register/finish` verifies attestation, checks AAGUID against MDS3 allowlist, stores credential. |
| FR-I-12 | `POST /webauthn/authenticate/begin` returns PublicKeyCredentialRequestOptions. |
| FR-I-13 | `POST /webauthn/authenticate/finish` verifies assertion, checks sign-count, audit-emits, issues OIDC tokens. |
| FR-I-14 | Conditional UI supported per WebAuthn L3. |
| FR-I-15 | Cross-platform + platform authenticators supported. |
| FR-I-16 | AAGUID allowlist refreshed weekly from FIDO MDS3 metadata. |

### D.3 SCIM 2.0 surface

| ID | Requirement |
|---|---|
| FR-I-20 | `GET /scim/v2/ServiceProviderConfig` advertises capabilities per RFC 7643. |
| FR-I-21 | `GET /scim/v2/Schemas` advertises supported attributes. |
| FR-I-22 | `POST /scim/v2/Users` creates a user. |
| FR-I-23 | `PUT /scim/v2/Users/{id}` updates a user. |
| FR-I-24 | `PATCH /scim/v2/Users/{id}` partial-updates a user. |
| FR-I-25 | `DELETE /scim/v2/Users/{id}` soft-deletes a user. |
| FR-I-26 | `POST /scim/v2/Groups` creates a group. |
| FR-I-27 | Per-tenant SCIM bearer token; rotates every 90 days. |
| FR-I-28 | SCIM endpoints serve only the tenant identified in the bearer; cross-tenant queries refused. |

### D.4 External IdP federation

| ID | Requirement |
|---|---|
| FR-I-30 | SAML 2.0 (SP-initiated + IdP-initiated) supported. |
| FR-I-31 | OIDC federation (upstream IdP issues OIDC; oyatie validates JWKS). |
| FR-I-32 | OAuth 2.1 (upstream) supported. |
| FR-I-33 | Apple Sign-In supported per Apple specs. |
| FR-I-34 | Google Sign-In / One-Tap supported. |
| FR-I-35 | KakaoTalk Login supported (pack-kr first-class). |
| FR-I-36 | LINE Login supported (pack-jp first-class). |
| FR-I-37 | WeChat OAuth supported where pack permits. |
| FR-I-38 | Naver Login supported (pack-kr first-class). |

### D.5 Step-up auth

| ID | Requirement |
|---|---|
| FR-I-40 | ACR classes: `routine, elevated, sensitive, critical` per `docs/standards/step-up-auth-classes.md`. |
| FR-I-41 | Each sensitive operation declares the minimum ACR; gate enforces. |
| FR-I-42 | Step-up flow re-prompts user for the missing factor. |
| FR-I-43 | Step-up valid for a bounded duration (default 5 minutes). |
| FR-I-44 | `critical` requires hardware token + 4-eye approval. |

### D.6 Age + minor handling per ADR-0292

| ID | Requirement |
|---|---|
| FR-I-50 | Age class derived from `birth_year` or upstream IdP attribute. |
| FR-I-51 | `<13` → COPPA refusal (US) or local-equivalent. |
| FR-I-52 | `14-17` → KOSA-tier defaults + parental notification. |
| FR-I-53 | `≥18` → adult; full feature access. |
| FR-I-54 | EU Member-State age-verification per pack overlay. |
| FR-I-55 | Minor-PII handling per ADR-0292 (no advertising, no profile-merging, no data-export-to-third-party). |

### D.7 Internal API for downstream µservices

| ID | Requirement |
|---|---|
| FR-I-60 | `POST /internal/v1/principal/resolve` accepts a Bearer token, returns full principal context. |
| FR-I-61 | `GET /internal/v1/jwks` (internal-only) returns signing keys + rotation metadata. |
| FR-I-62 | `POST /internal/v1/step-up/grant` issues a step-up token after factor presentation. |

### D.8 provider-BYOK + tenant-owned IdP

| ID | Requirement |
|---|---|
| FR-I-70 | Tenant MAY configure upstream IdP (Okta, Entra, Workspace, Auth0) as primary. |
| FR-I-71 | Upstream IdP secrets (SAML certs, OIDC client secrets) stored as SecretReference per ADR-0255. |
| FR-I-72 | Disconnect flow: 30-day grace; old tokens still valid; new auths refused after grace. |

### D.9 Audit + compliance

| ID | Requirement |
|---|---|
| FR-I-80 | Every event audit-chain sealed per ADR-0028 within 1s. |
| FR-I-81 | DSAR-export endpoint surfaces user identity data per pack retention rules. |
| FR-I-82 | DSAR-erasure endpoint tombstones PII per pack rules. |

---

## E. Non-functional Requirements

### E.1 Performance budgets

| Metric | P50 | P95 | P99 | Notes |
|---|---|---|---|---|
| OIDC token issuance | ≤25 ms | ≤60 ms | ≤80 ms | Zitadel-backed; Postgres event-store insert + signing |
| OIDC token verification (JWKS cached in-process) | ≤500 µs | ≤1.5 ms | ≤2 ms | stateless verify |
| WebAuthn register/finish | ≤80 ms | ≤200 ms | ≤250 ms | attestation parse + AAGUID validation + Postgres insert |
| WebAuthn authenticate/finish | ≤30 ms | ≤80 ms | ≤100 ms | signature verify + sign-count check |
| SCIM POST /Users | ≤150 ms | ≤400 ms | ≤500 ms | Zitadel admin API + audit-emit |
| Step-up ACR grant flow | ≤3 s | ≤6 s | ≤8 s | UX-bound; redirect + ceremony |
| JWKS endpoint serve | ≤2 ms | ≤6 ms | ≤8 ms | in-process cached |
| SAML response verify | ≤30 ms | ≤80 ms | ≤120 ms | XML signature + assertion validate |
| OIDC federation callback | ≤50 ms | ≤150 ms | ≤300 ms | upstream JWKS fetch (cached) + verify |

(Evidence: modeling notes `docs/performance-budgets/identity-token-issuance.md` + `docs/performance-budgets/identity-webauthn-budget.md` to be authored M01.)

### E.2 Availability

| Surface | Target |
|---|---|
| OIDC token issuance | 99.99% monthly |
| OIDC JWKS | 99.999% (downstream µservices cache; brief unavailability tolerable) |
| WebAuthn authenticate | 99.99% |
| SCIM endpoints | 99.95% (B2B; eventually consistent acceptable) |
| Step-up auth | 99.95% |

### E.3 Scalability

- Per-cell: 50,000 token issuances/s sustained; bursts to 250,000/s.
- Per-tenant rate limits: 1,000 token issuances/s default; per-tenant override.
- Postgres + Citus sharded by `tenant_id` for SCIM + audit.
- Token signing horizontal: stateless signers; signing keys in OpenBao.
- Active-active per-pack within region.

### E.4 Security

- Phishing-resistant first factor by default per ADR-0188.
- Tokens NEVER logged in plaintext; `Classified<Token>` wrapper.
- JWT signing keys in OpenBao; HSM-backed in regulated packs.
- Per-tenant SCIM bearers rotate every 90 days.
- JWKS rotation every 24h; signing-key rotation every 90 days.
- Constant-time bearer comparison for SCIM auth.
- OWASP A07 (Auth Failures) hardening: rate-limited login (edge), account lockout (5 in 5min → 15min cool-off), NIST SP 800-63B password policy when password fallback used, MFA-fatigue protection (push-with-number-matching only).
- STRIDE per endpoint:
  - **T** (token replay) → JWT `jti` cache + token-revocation deny-list at edge.
  - **I** (info disclosure) → no PII in error responses; constant error shape.
  - **R** (repudiation) → audit-chain seal per ADR-0028 every grant + signin + step-up.
  - **D** (DoS) → edge rate-limit per ADR-0191; per-tenant token issuance quota.
  - **E** (priv escalation) → ACR floor per ADR-0189; Cedar gate on every grant.

### E.5 Audit + compliance

- Every event emits `AuditEvent` Merkle-chained per (tenant, period) per ADR-0028.
- Retention per pack: KR-PIPA Art. 30 (≥1y), HIPAA §164.316(b)(2) (6y), GDPR Art. 30 (purpose-bounded), PCI-DSS §10.5.1 (≥1y, 3mo immediately available).
- SOC 2 CC6.1 (logical access) + ISO 27001 A.9 (access control) + PCI-DSS v4.0 §8 (identify users).

### E.6 Data residency

- Per-pack identity data sovereignty per ADR-0179.
- No cross-pack identity replication.
- Per-pack Zitadel instances; per-pack JWKS issuers (`https://identity-<pack>.oyatie.com`).

### E.7 DR posture

| Field | Value |
|---|---|
| ADR | ADR-0343 |
| Target | RTO 1800 s, RPO 0 s for identity state, matching `manifest.json#dr`; legacy `manifest.json#rpo_rto` remains stricter for realtime tenant-class paths at RTO 30 s / RPO 0 s. |
| Compliance-pack floor | HIPAA floor RTO 3600 s / RPO 300 s, PCI-DSS L1 floor RTO 86400 s / RPO 3600 s, SOC2-T2 floor RTO 14400 s / RPO 900 s; identity's manifest target is stricter at 1800 s / 0 s. |
| Failover runbook | `runbooks/idp-failover-drill.md`, with recovery-key and IP-block incident runbooks as adjacent identity incident paths. |
| Multi-region active-active | Yes within the same compliance pack and cell family; cross-pack identity replication remains forbidden. |
| WHY | A tenant-visible login, JWKS, SCIM, and step-up outage blocks every product surface, so the DR posture preserves session continuity and authz validation rather than merely restoring an admin control plane. |

### E.8 Capacity model

| Field | Value |
|---|---|
| ADR | ADR-0340, with pod runtime tier declared by ADR-0338. |
| Per-tenant baseline | `manifest.json#capacity_model`: 0.18 vCPU, 256 MiB RAM, 2 GB storage, and connections `{valkey: 4, postgres: 2, outbound_http: 5}` per tenant. Default quotas from `capacity-model.md` keep `/oauth/v2/token` at 100 rps, `/oauth/v2/userinfo` at 200 rps, `/webauthn/*` at 50 rps, SCIM POST at 100 rps, and SCIM PATCH at 200 rps before override. |
| Scaling dimension | `per_request`, matching `manifest.json#capacity_model.scaling_dimension`; per-seat billing remains a FinOps dimension, not the scaling driver. |
| Cell placement class | Tier-0 service criticality per manifest `criticality_tier=T0`; runtime tier is ADR-0338 Tier-1 because `manifest.json#pod_runtime_tier=1` and identity handles tenant identity data, HSM-backed keys, and OpenBao SecretReference material. |
| Autoscaling boundaries | Per pack floor: 3 Zitadel pods. Year-5 projection cap: 20 Zitadel pods per pack plus tenant-sharded Postgres before raising per-tenant quotas. HPA target: 70% CPU, 80% memory, 60 s scale-out cooldown. |
| WHY | The model protects login surges, SCIM bulk imports, and WebAuthn ceremonies while limiting a single tenant's auth burst from starving other tenants. |

### E.9 Sustainability + cost attribution

| Field | Value |
|---|---|
| ADR | ADR-0344 |
| Per-call emission claim | Every identity audit row, including token, SCIM, WebAuthn, IdP binding, and key-rotation events, must carry `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, and `region` beside the Merkle audit fields. |
| Carbon-aware routing | No for interactive authentication, step-up, emergency access, and HIPAA-regulated identity flows. Yes only for low-urgency background jobs such as SCIM backfill, HRIS polling, AAGUID refresh, and audit DLQ replay when residency and freshness stay inside policy. |
| Tenant transparency surface | Identity exposes per-seat and per-usage line items through the tenant billing view and FinOps portal, keyed by the `paid_billing_components_emitted` values in the manifest. |
| WHY | CSRD, SB-253, SEC climate-disclosure posture, and customer trust require authentication cost and emissions to be visible without letting carbon routing weaken access continuity. |

### E.10 API versioning posture

| Field | Value |
|---|---|
| ADR | ADR-0342 |
| Public API version model | Date carrier triplet: `Oyatie-Version: YYYY-MM-DD`, `/v/YYYY-MM-DD/...` URL prefix where exposed, and proto3 `oyatie_version` field. |
| SDK semver model | Identity SDKs use `major.minor.patch`; API compatibility remains pinned to the date carrier. |
| Support window | Last N=3 public versions supported for >=180 days. |
| Per-tenant pinning | Yes for B2B OIDC/SAML/SCIM tenants and regulated passkey rollout cohorts. |
| Internal-mesh exemption | Yes. Direct gRPC inside the mesh remains governed by ADR-0145 and does not require public URL date prefixes. |

---

## F. UX Flows

### F.1 Passkey-first consumer signup

```
[User taps Sign Up]
       |
       v
[Choose IdP: KakaoTalk / Google / Apple / Email / Phone]
       |
       v (e.g., KakaoTalk)
[KakaoTalk OAuth → user confirms in KakaoTalk app]
       |
       v
[oyatie callback: validates upstream token]
       |
       v
[Create personal tenant __personal__/<user_id>]
       |
       v
[Prompt: Enroll passkey?]
       |
       v (yes)
[WebAuthn /register/begin → browser/OS ceremony]
       |
       v
[/register/finish: attestation verify + AAGUID check + store]
       |
       v
[Signin complete; OIDC token issued]
       |
       v
[Welcome screen + onboarding wizard]
```

### F.2 B2B SAML SSO sign-in

```
[Employee visits oyatie work app]
       |
       v
[Enter tenant slug: acme]
       |
       v
[Tenant config: SAML upstream IdP=okta]
       |
       v
[Redirect to Okta /sso?RelayState=...]
       |
       v
[Okta authenticates user (with Okta MFA)]
       |
       v
[Okta posts SAML assertion to oyatie ACS]
       |
       v
[Validate assertion signature + audience + conditions]
       |
       v
[Extract subject + groups + attributes]
       |
       v
[Map groups → oyatie roles]
       |
       v
[Issue oyatie OIDC token]
       |
       v
[Redirect to original target]
```

### F.3 SCIM user provisioning

```
[Okta hire event → SCIM POST /scim/v2/Users]
       |
       v
[Validate per-tenant bearer token]
       |
       v
[Parse SCIM payload; extract userName, emails, name, active]
       |
       v
[Create Zitadel user]
       |
       v
[Apply group memberships → role assignments]
       |
       v
[Audit chain: IdentityUserProvisioned]
       |
       v
[Return SCIM 201 + Location]
```

### F.4 Step-up to acr=critical

```
[Admin clicks "Delete tenant"]
       |
       v
[Backend Cedar check: needs acr=critical; current=elevated → 401 + step-up_required]
       |
       v
[UI: Step-up required: enter passkey + IT-approval]
       |
       v
[User taps passkey → WebAuthn ceremony]
       |
       v
[Second admin notified via messenger]
       |
       v
[Second admin approves]
       |
       v
[Step-up token issued (5-min validity)]
       |
       v
[Retry delete-tenant → succeeds with audit chain]
```

### F.5 Account recovery flow

```
[User clicks "Lost access"]
       |
       v
[Email + recovery code OR ID-doc upload]
       |
       v
[24h cooldown; user notified by all means]
       |
       v
[After cooldown, recovery completes]
       |
       v
[Sign in granted with acr=elevated only]
       |
       v
[Critical ops still require step-up]
```

### F.6 Minor signup with KOSA tier

```
[User taps Sign Up; enters birth year 2010 → age 16]
       |
       v
[ADR-0292: KOSA tier]
       |
       v
[Parent contact required]
       |
       v
[Parent notified by email + SMS]
       |
       v
[Account created with privacy defaults]
       |
       v
[On 18th birthday: graduation event auto-runs]
```

### F.7 OAuth device-code for CLI

```
[oya CLI runs `oya login`]
       |
       v
[POST /oauth/device_authorization]
       |
       v
[CLI prints: "Open https://oyatie.com/device and enter code ABCD-EFGH"]
       |
       v
[User opens URL in browser; enters code; signs in]
       |
       v
[CLI polls /oauth/token until granted]
       |
       v
[CLI receives token; saved to ~/.oya/credentials]
```

### F.8 Impossible-travel detection

```
[Signin from Seoul at T]
       |
       v
[Signin from London at T+10min]
       |
       v
[Distance ~ 8800km; minimum travel time > 8h → impossible]
       |
       v
[Risk engine score = HIGH]
       |
       v
[Step-up required + alert to user via all comm channels]
       |
       v
[If user does not respond in 5 min, sessions invalidated]
```

---

## G. Success Metrics

### G.1 Latency

- P50 token issuance: ≤25 ms.
- P99 token issuance: ≤80 ms.
- P50 WebAuthn authenticate: ≤30 ms.
- P99 WebAuthn authenticate: ≤100 ms.

### G.2 Throughput

- Sustained 50,000 token issuances/s per cell.
- Sustained 10,000 WebAuthn ceremonies/s.

### G.3 Conversion + retention

- Consumer signup completion ≥ 85%.
- Passkey enrollment rate (offered) ≥ 70%.
- B2B SSO uptake within 90 days of tenant onboarding ≥ 80%.
- Account-recovery success ≥ 95% (genuine users).

### G.4 Security

- Account-takeover incidents ≤ 0.001% of monthly active accounts.
- Phishing-resistant credential coverage ≥ 90% of B2C; 100% of B2B critical-ops.
- MFA-fatigue attack-success rate = 0.

### G.5 Support + business

- Tickets per 1k sign-ins ≤ 0.3.
- NPS (B2C) ≥ 65.
- NPS (B2B admin) ≥ 55.
- DAU/MAU on auth surface tracks closely with overall product DAU/MAU.

---

## H. Compliance Impact

| Pack | Standards |
|---|---|
| pack-us | SOC 2 Type II; CCPA/CPRA; COPPA |
| pack-us-healthcare | HIPAA; identity events as required by §164.308(a)(5)(ii)(C) |
| pack-eu | GDPR; eIDAS for high-assurance auth; DSA; AI Act Annex III for risk-engine if classified |
| pack-uk | UK GDPR; FCA SCA requirements when payment-adjacent |
| pack-kr | PIPA; KR-FSS where finance-adjacent; ISMS-P |
| pack-jp | APPI |
| pack-sg | PDPA + MAS |
| pack-au | Privacy Act 1988 |
| pack-br | LGPD |
| pack-kosa | KOSA 14-17 tier rules |
| pack-coppa-refuse | COPPA <13 refusal |

Compliance evidence emission:

- Per-event audit-chain (ADR-0028).
- DSAR-export + erasure flows (GDPR Art. 15 + 17; KR-PIPA Art. 35 + 36).
- Annual SOC 2 Type II + ISO 27001 attestation.
- Quarterly access-review reports.

---

## I. Open Questions

| # | Question | Owner | Target ADR / Date |
|---|---|---|---|
| 1 | Passwordless-first vs password-fallback by default? Bias: passwordless default; password allowed only with admin opt-in. | council-privacy | M01 |
| 2 | KR-FSS-mandated SMS-OTP retention vs NIST SP 800-63B SMS-deprecation: pack-specific override? | council-privacy + ops-security | M01 |
| 3 | Cross-region active-active failover for identity within a pack: 1 region active, 2nd standby; or 2 region active? | ops-sre + axis-identity | M02 |
| 4 | OAuth scope catalog: enumerated central registry vs per-µservice? | council-architecture | M01 |
| 5 | DPoP enforcement on legacy clients: grace period? | axis-identity + council-architecture | M02 |
| 6 | Minor-PII-data export: refused even via DSAR-export, or allowed only to parent? | council-privacy + legal | M01 |
| 7 | EU AI Act Annex III risk-engine treatment: classify or refuse? | council-privacy + axis-identity | M02 |
| 8 | Federation Sign-In with Web3 (SIWE): in scope or out? | council-architecture | M04 |

---

## J. Out of Scope

1. **Mobile-device SDK** — out of scope at M01; future `microservices/mobile-shell` µservice will host.
2. **Self-built crypto** — explicitly forbidden; use Zitadel + webauthn-rs.
3. **Per-µservice IdP** — one IdP substrate fleet-wide.
4. **Cross-pack identity replication** — sovereign-residency forbidden.
5. **Legacy SAML 1.1 / WS-Federation** — rejected; obsolete.
6. **Behavioral biometrics** — out of scope at M01; future enhancement.
7. **Identity-proofing / KYC for B2C** — out of scope; lives in `connector` µservice.
8. **PAM (privileged access management) for prod infra** — out of scope; lives in `ops-pam` future µservice.
9. **Decentralized identity (DID, Verifiable Credentials)** — out of scope at M01; M05+ exploration.

---

## K. Bounded Contexts (BC tree)

Per ADR-0105 13-value layer enum + ADR-0106 usecase rename:

| BC | Crate family | Purpose |
|---|---|---|
| `oidc-issuer` | `oya-identity-oidc-issuer-{kernel,domain,usecase,api,adapter,adapter-zitadel,rest,sdk,app}` | OIDC token issuance + JWKS + revocation |
| `webauthn-relying-party` | `oya-identity-webauthn-relying-party-{kernel,domain,usecase,api,adapter,adapter-webauthn-rs,rest,worker,sdk,app}` | WebAuthn L3 register + authenticate |
| `scim-server` | `oya-identity-scim-server-{kernel,domain,usecase,api,adapter,adapter-zitadel,rest,sdk,app}` | SCIM 2.0 inbound endpoint |
| `hris-adapter` | `oya-identity-hris-adapter-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | Non-SCIM HRIS adapter contract |
| `step-up-orchestrator` | `oya-identity-step-up-orchestrator-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | Step-up ACR grant flow |
| `external-idp-federation` | `oya-identity-external-idp-federation-{kernel,domain,usecase,api,adapter,adapter-saml,adapter-oidc,adapter-apple,adapter-google,adapter-kakao,adapter-line,adapter-wechat,adapter-naver,rest,sdk,app}` | Upstream IdP federation |
| `audit-emitter` | `oya-identity-audit-emitter-{kernel,domain,usecase,api,adapter,worker,sdk}` | Bridge to audit-chain |
| `zitadel-instance-controller` | `oya-identity-zitadel-instance-controller-{kernel,domain,usecase,api,adapter,worker,app}` | Per-pack Zitadel lifecycle |
| `risk-engine` | `oya-identity-risk-engine-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | Impossible-travel + password-spray + MFA-fatigue detection |
| `recovery-flow` | `oya-identity-recovery-flow-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | Account recovery + 24h cooldown |
| `age-assurance` | `oya-identity-age-assurance-{kernel,domain,usecase,api,adapter,sdk}` | COPPA / KOSA / EU age-verification |
| `principal-resolver` | `oya-identity-principal-resolver-{kernel,domain,usecase,api,adapter,sdk}` | Multi-context principal resolution |

Total crates: ~85 across 12 BCs.

---

## L. Integration Surface

### L.1 Workflow events produced

| Event type | Trigger | Consumed by |
|---|---|---|
| `identity.user.provisioned` | SCIM POST | tenancy, downstream provisioning |
| `identity.user.suspended` | SCIM PATCH active=false | tenancy, payments (suspend charges) |
| `identity.user.deprovisioned` | SCIM DELETE | tenancy, every µservice (revoke) |
| `identity.signin.succeeded` | sign-in success | audit-chain, risk-engine |
| `identity.signin.failed` | sign-in failure | risk-engine |
| `identity.webauthn.registered` | passkey enroll | audit-chain |
| `identity.step_up.granted` | step-up success | audit-chain, operation gate |
| `identity.recovery.initiated` | recovery start | audit-chain, user notification |
| `identity.minor.signup.refused` | COPPA refusal | parent notification |
| `identity.minor.kosa.activated` | KOSA tier activated | parent notification |
| `identity.idp.federated` | upstream IdP bound | tenancy, ops |
| `identity.token.revoked` | revocation | api-gateway deny-list |
| `identity.signing_key.rotated` | key rotation | api-gateway JWKS refresh |

### L.2 Workflow events consumed

| Event type | Produced by | Action |
|---|---|---|
| `tenant.onboarded` | tenancy | provision per-tenant SCIM bearer; default IdP config |
| `tenant.deleted` | tenancy | cascade-delete users + sessions |
| `compliance.pack.attached` | compliance | reload Cedar fragments; update age-assurance rules |
| `dsar.erasure.requested` | governance | cascade-DSAR; tombstone identity PII |

### L.3 Ontology writes

| Object Type | Written by BC |
|---|---|
| `identity::User` | scim-server |
| `identity::Credential` | webauthn-relying-party |
| `identity::Session` | oidc-issuer |
| `identity::SignInEvent` | oidc-issuer + audit-emitter |
| `identity::StepUpGrant` | step-up-orchestrator |
| `identity::IdpFederation` | external-idp-federation |

### L.4 Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `tenancy::Tenant` | oidc-issuer | tenant config + jurisdiction pack |
| `compliance::CompliancePack` | age-assurance | applicable packs for the tenant |

---

## M. Acceptance criteria

| ID | Criterion | Verification |
|---|---|---|
| AC-I-01 | OIDC token issuance P99 ≤ 80ms | k6 load |
| AC-I-02 | WebAuthn authenticate P99 ≤ 100ms | k6 load |
| AC-I-03 | SCIM provisioning end-to-end ≤ 500ms | nextest |
| AC-I-04 | Step-up grant audit-chain seal | nextest |
| AC-I-05 | Cross-tenant SCIM refused | nextest + Cedar gate |
| AC-I-06 | COPPA <13 refusal works | nextest |
| AC-I-07 | KOSA tier defaults applied | nextest |
| AC-I-08 | DSAR-export ZIP signed | e2e |
| AC-I-09 | DSAR-erasure tombstones PII | e2e |
| AC-I-10 | Per-pack residency (no cross-pack federation) | nextest |
| AC-I-11 | JWKS rotation overlap window works | e2e |
| AC-I-12 | Token revocation propagates to api-gateway ≤ 5 min | e2e |
| AC-I-13 | Refresh-token rotation invalidates on reuse | nextest |
| AC-I-14 | Impossible-travel detection triggers step-up | e2e |
| AC-I-15 | Password-spray detection blocks IP block | e2e |
| AC-I-16 | MFA-fatigue mitigation: number-matching required | nextest |
| AC-I-17 | SAML response signature verification | nextest |
| AC-I-18 | OIDC federation callback validation | nextest |
| AC-I-19 | DPoP enforcement | nextest |
| AC-I-20 | Minor 18th-birthday graduation | scheduled job test |

---

## N. Performance evidence

### N.1 Modeling notes

- `docs/performance-budgets/identity-token-issuance.md` (TBD M01) — decomposes 80ms P99 into: Cedar tenant-context check (5ms), Zitadel event-store write (40ms), signing (5ms), audit-emit (5ms), response render (5ms), buffer (20ms).
- `docs/performance-budgets/identity-webauthn-budget.md` (TBD M01) — decomposes 100ms P99 authenticate into: assertion fetch (5ms), Ed25519/ECDSA verify (10ms), sign-count check (10ms), Postgres update (20ms), audit-emit (10ms), response (5ms), buffer (40ms).

### N.2 Hyperscaler benchmark comparisons

- **Okta**: P50 token ~50ms, P99 ~150ms (public).
- **Auth0**: P50 ~80ms, P99 ~250ms.
- **AWS Cognito**: P50 ~120ms (cold), P99 ~400ms.
- **Microsoft Entra**: P50 ~40ms, P99 ~100ms.
- **oyatie target**: P50 ≤ 25ms, P99 ≤ 80ms. Above all listed competitors; achievable via Zitadel in-cell + signing keys in-process.

---

## O. Migration + rollout

### O.1 M01 ship plan

- Week-1 to Week-4: Zitadel deployment per pack (pack-us, pack-eu, pack-kr at M01).
- Week-5 to Week-8: OIDC issuer + JWKS + WebAuthn relying party.
- Week-9 to Week-12: SCIM server + external IdP federation (SAML + OIDC).
- Week-13 to Week-16: Step-up orchestrator + age-assurance + recovery flow.
- Week-17 to Week-20: HRIS adapter + risk engine + audit emitter.
- Week-21 to Week-22: E2E + load + chaos tests.
- Week-23 to Week-26: M01 ship.

### O.2 M02+ expansion

- Additional packs (pack-jp, pack-sg, pack-au, pack-br) per ADR-0179 staging.
- More upstream IdPs (Auth0, OneLogin, Ping).
- Behavioral-biometric step-up (M03).
- DPoP universal enforcement (M03).
- DID / VC exploration (M05+).

### O.3 Sunset + deprecation

- No deprecation at M01 (greenfield).
- Future deprecations follow `feedback_no_silent_regression` (18-month notice).

---

## P. Cross-Slice References (to be added)

- **Slice ADR-author** — link to ADRs 0187, 0188, 0189, 0190, 0191 + any new identity-specific ADRs.
- **Slice runbook-author** — `microservices/identity/runbooks/key-rotation.md`, `account-recovery-incident.md`, `scim-provisioning-incident.md`, `impossible-travel-alert.md`, `idp-upstream-failover.md`.
- **Slice spec-author** — `/specs/microservices/identity.json` for token + SCIM + WebAuthn payload schemas.
- **Slice user-story-bank** — extend `b2c-consumer-surfaces.md` and `b2b-work-surfaces.md` with identity stories referencing this PRD.
- **Slice testing-strategy** — `microservices/identity/testing-strategy.md` for E2E, fuzz (token manipulation), property-based session-state-machine tests, IdP-mock harnesses.
- **Slice synthesis** — keystone-bundle synthesis doc.
- **Slice memory** — `feedback_identity_substrate_2026_05_20.md` capture.

---

## Q. Threat model summary

### Q.1 STRIDE per BC

| BC | Spoofing | Tampering | Repudiation | Info disclosure | DoS | Elevation |
|---|---|---|---|---|---|---|
| `oidc-issuer` | Issuer-impersonation: JWKS pinning, DNSSEC, certificate transparency | Token tampering: signing key compromise → emergency rotation; integrity check on all reads | All grants audit-chained; per-event provenance | Token payload + JWKS public; no PII in token beyond `sub` (opaque); error responses constant-shape | Rate-limit per IP + per-tenant; circuit-breaker on Zitadel backend | ACR-floor on every operation; Cedar gate refuses elevation without step-up |
| `webauthn-relying-party` | RP-ID spoofing: validate origin per spec; reject mismatched origin | Attestation tampering: signature verify; AAGUID allowlist | All ceremonies audit-chained | Public-key registry; no biometric leaves device | Per-IP register-attempt rate-limit | Sign-count regression refuses authenticate |
| `scim-server` | SCIM-bearer leak: 90-day rotation; constant-time compare; audit-chain emission | Payload tampering: bearer auth ensures integrity | Per-tenant audit | Tenant-scoped responses only; cross-tenant refused | Per-tenant rate-limit; backpressure to Zitadel | Per-tenant scope; no cross-tenant admin actions |
| `step-up-orchestrator` | Step-up bypass: ACR claim is server-issued + audit-chained | Step-up token includes session-binding | Audit-chain emission on grant | Step-up token short-lived (5min) | Per-user step-up-rate-limit | Self-grant of step-up refused; 4-eye for critical |
| `external-idp-federation` | Upstream IdP impersonation: SAML cert pinning; OIDC issuer pinning | SAML assertion signature + replay window | Per-federation audit | Minimal-scope OIDC; attribute-disclosure Cedar-gated | Upstream IdP failover within pack | Upstream group → role mapping admin-gated |
| `recovery-flow` | Social-engineering: 24h cooldown; multi-channel notification | Recovery action audit-chained | Per-recovery audit | Recovery codes never logged | Per-user recovery-rate-limit | Recovery yields only `acr=elevated` |
| `risk-engine` | Score-tampering: server-side compute | Risk-event signed; tamper detect | Per-event audit | Score not exposed to user | N/A | High-score auto-step-up |

### Q.2 Common attack scenarios + mitigations

| Attack | Mitigation |
|---|---|
| Phishing for password | Passkey-first; password-fallback configurable per tenant |
| SIM-swap → SMS OTP bypass | SMS-OTP region-gated; passkey preferred |
| Pass-the-cookie (Lapsus$) | DPoP token-binding |
| MFA push-fatigue | Number-matching mandatory on push |
| Credential stuffing | Per-IP + per-account rate-limit + lockout |
| Token theft via XSS | HttpOnly cookies + SameSite=Strict + CSP header |
| OAuth redirect-URI tampering | Exact-match redirect-URI; refuse partial-match |
| SAML signature wrapping | xmldsig validation per spec; reject mixed-content assertions |
| OIDC issuer impersonation | JWKS pinning per tenant config |
| SCIM bearer leak | 90-day rotation; constant-time compare |
| Lost device | Multi-passkey + YubiKey + recovery flow; revoke by other credential |
| Internal-admin compromise | 4-eye approval for critical ops; audit-chain |

### Q.3 Compliance evidence outputs

| Standard | Evidence |
|---|---|
| SOC 2 CC6.1 | per-user audit log of access; access-review report per quarter |
| ISO 27001 A.9 | access-control policy doc + per-quarter review |
| PCI-DSS v4.0 §8 | unique user IDs + MFA per §8.4 + lockout per §8.3.4 |
| GDPR Art. 32 | technical + organisational measures; pseudonymization where applicable |
| KR-PIPA Art. 29 | safeguards for personal information; per-event audit |
| HIPAA §164.308(a)(5)(ii)(C) | log-in monitoring procedures |
| KOSA | per-minor audit trail; parent visibility report |

---

## R. Sample Cedar policies

```cedar
// Refuse <13 signups
forbid (
  principal,
  action == Action::"identity::signup",
  resource is identity::SignupRequest
) when {
  resource.age_class == "under_13"
};

// KOSA tier: require parental notification
permit (
  principal,
  action == Action::"identity::signup",
  resource is identity::SignupRequest
) when {
  resource.age_class == "14_17" &&
  resource.parental_notification_dispatched == true
};

// Step-up for critical ops
forbid (
  principal,
  action == Action::"identity::critical_op",
  resource
) unless {
  principal.acr == "critical" &&
  principal.step_up_recent_within_5min == true
};

// Cross-tenant SCIM refused
forbid (
  principal is identity::ScimClient,
  action == Action::"scim::write",
  resource is identity::User
) when {
  principal.tenant_id != resource.tenant_id
};

// Self-grant of higher role refused
forbid (
  principal,
  action == Action::"identity::role_grant",
  resource is identity::User
) when {
  principal.subject == resource.subject &&
  resource.target_role.tier > principal.role.tier
};
```

---

## S. Sample OIDC discovery response

```json
{
  "issuer": "https://identity-kr.oyatie.com",
  "authorization_endpoint": "https://identity-kr.oyatie.com/oauth/authorize",
  "token_endpoint": "https://identity-kr.oyatie.com/oauth/token",
  "userinfo_endpoint": "https://identity-kr.oyatie.com/oauth/userinfo",
  "jwks_uri": "https://identity-kr.oyatie.com/oauth/jwks",
  "registration_endpoint": "https://identity-kr.oyatie.com/oauth/register",
  "scopes_supported": ["openid", "profile", "email", "tenant_id", "acr", "purpose", "data_class"],
  "response_types_supported": ["code", "id_token", "id_token token"],
  "grant_types_supported": ["authorization_code", "refresh_token", "urn:ietf:params:oauth:grant-type:device_code", "urn:ietf:params:oauth:grant-type:token-exchange"],
  "subject_types_supported": ["public", "pairwise"],
  "id_token_signing_alg_values_supported": ["EdDSA", "ES256", "RS256"],
  "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post", "private_key_jwt", "none"],
  "code_challenge_methods_supported": ["S256"],
  "acr_values_supported": ["routine", "elevated", "sensitive", "critical"],
  "dpop_signing_alg_values_supported": ["ES256", "EdDSA"]
}
```

---

## T. Sample ID token claims

```json
{
  "iss": "https://identity-kr.oyatie.com",
  "sub": "user_01HZX2N5Q3K8R7M6V4P9W2Y3F1",
  "aud": "messenger.oyatie.com",
  "exp": 1747765931,
  "iat": 1747762331,
  "nbf": 1747762331,
  "tenant_id": "__personal__/user_01HZX2N5Q3K8R7M6V4P9W2Y3F1",
  "acr": "elevated",
  "amr": ["webauthn", "kakao"],
  "purpose": "messenger.send",
  "data_class": "personal",
  "age_class": "adult",
  "jurisdiction_code": "KR",
  "compliance_packs": ["pack-kr", "pack-kr-pipa"],
  "kakao_sub": "...",
  "step_up_valid_until": null,
  "session_id": "sess_01HZX2N5..."
}
```

---

## U. Bounded Context details

### U.1 `oidc-issuer` BC

The OIDC issuer is the hottest BC by request volume (every API request to every µservice indirectly hits the JWKS endpoint via cached lookups). It runs on Zitadel under the hood (per ADR-0187); oyatie's substrate wraps Zitadel with:

- Cedar-gated admission to token issuance (refuse if Cedar denies the principal-context).
- Per-tenant rate-limit (defense in depth above api-gateway).
- Audit-chain emission on every grant.
- Token claim enrichment: tenant_id, acr, purpose, data_class, age_class, jurisdiction_code, compliance_packs.

Adapter `oya-identity-oidc-issuer-adapter-zitadel` translates between Zitadel's native token shape and oyatie's claim schema.

### U.2 `webauthn-relying-party` BC

Implements WebAuthn L3 server-side using `webauthn-rs` (Rust crate). Key behaviours:

- FIDO MDS3 metadata refresh: weekly cron job pulls FIDO Alliance metadata; updates AAGUID allowlist.
- AAGUID validation: refuse credentials from authenticators not on the allowlist (configurable per tenant).
- caBLE: cross-device authentication via BLE supported (iOS/Android can be a 2FA for desktop).
- Conditional UI: returned options support browser conditional-UI flow.
- Backup-eligible credentials: detected via attestation; surfaced in dashboard.

### U.3 `scim-server` BC

Implements SCIM 2.0 per RFC 7643 + RFC 7644:

- `/v2/Users` (create / read / update / patch / delete).
- `/v2/Groups`.
- `/v2/Schemas` advertises supported attributes.
- `/v2/ServiceProviderConfig` advertises capabilities.
- `/v2/ResourceTypes`.
- Filter syntax: limited subset of RFC 7644 §3.4.2.2 (eq, ne, sw, ew, co, gt, lt + and/or).
- Per-tenant bearer authentication.

Common SCIM-client compatibility: Okta, Microsoft Entra, Google Workspace, JumpCloud, OneLogin.

### U.4 `external-idp-federation` BC

Sub-adapters:

- `-adapter-saml` — SAML 2.0 with xmldsig + xmlenc; SP-initiated + IdP-initiated.
- `-adapter-oidc` — OIDC + OAuth 2.1 federation.
- `-adapter-apple` — Apple Sign-In (OIDC variant).
- `-adapter-google` — Google Sign-In + One-Tap.
- `-adapter-kakao` — KakaoTalk OAuth 2.0.
- `-adapter-line` — LINE Login OAuth 2.0.
- `-adapter-wechat` — WeChat OAuth 2.0.
- `-adapter-naver` — Naver Login OAuth 2.0.

Per-IdP capability matrix:

| IdP | Auth | SCIM-equivalent | Group sync | Refresh |
|---|---|---|---|---|
| Okta | SAML+OIDC | SCIM | Yes | Yes |
| Microsoft Entra | SAML+OIDC | SCIM | Yes | Yes |
| Google Workspace | OIDC | Directory API | Yes | Yes |
| Apple Sign-In | OIDC | N | N | Yes (limited) |
| Google Sign-In (consumer) | OIDC | N | N | Yes |
| KakaoTalk | OAuth 2.0 | N | N | Yes |
| LINE | OAuth 2.0 | N | N | Yes |
| WeChat | OAuth 2.0 | N | N | Yes |
| Naver | OAuth 2.0 | N | N | Yes |

### U.5 `step-up-orchestrator` BC

Manages ACR class transitions:

```
routine: existing session
elevated: re-auth within session (re-presenting current factor)
sensitive: fresh factor (password + TOTP / passkey)
critical: hardware token + 4-eye approval
```

Per `docs/standards/step-up-auth-classes.md`, each operation declares its required ACR; gate refuses or triggers step-up.

### U.6 `risk-engine` BC

Detects:

- Impossible-travel (distance / time-since-last-signin).
- Velocity (sign-in attempts per minute per account or IP).
- Password-spray (one password across many accounts from same IP block).
- MFA-fatigue (rapid push request attempts).
- Geo-mismatch (billing country vs IP country).

Outputs `risk_score 0-100`; high score auto-triggers step-up.

### U.7 `recovery-flow` BC

Implements Account-Recovery with:

- 24h cooldown (KR-PIPA-compliant).
- Multi-channel notification (email + SMS where regulator allows + push).
- ID-document upload option (heavy weight; admin review).
- Recovery codes (one-time pads; printed + downloaded at enroll).
- Limited grant: recovery yields `acr=elevated` only; `critical` ops still need step-up.

### U.8 `age-assurance` BC

Per ADR-0292 minor doctrine:

- Birth-year input or upstream-IdP-attribute.
- Cedar policies: `identity::age_assurance::coppa_refuse`, `identity::age_assurance::kosa_minor_tier`, `identity::age_assurance::eu_member_state`, `identity::age_assurance::kr_youth`.
- Parental notification dispatch.
- 18th-birthday graduation scheduled job.

### U.9 `principal-resolver` BC

Resolves a Bearer token into the full principal context:

- JWKS verify (cached).
- Tenant context extraction.
- Compliance packs lookup.
- ACR class.
- Multi-context (personal + work) determined by tenant_id prefix (`__personal__/...` vs `<tenant_slug>`).

---

## V. Sample SCIM request/response

### V.1 SCIM POST /Users (Okta hire)

```http
POST /scim/v2/Users HTTP/1.1
Host: identity-us.oyatie.com
Authorization: Bearer <per-tenant-scim-bearer>
Content-Type: application/scim+json

{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
  "userName": "alice@acme.com",
  "name": { "givenName": "Alice", "familyName": "Lee" },
  "emails": [{ "value": "alice@acme.com", "type": "work", "primary": true }],
  "active": true,
  "groups": [{ "value": "engineering", "display": "Engineering" }],
  "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User": {
    "employeeNumber": "E12345",
    "department": "Platform",
    "manager": { "value": "M001" }
  }
}
```

### V.2 SCIM response

```http
HTTP/1.1 201 Created
Content-Type: application/scim+json
Location: https://identity-us.oyatie.com/scim/v2/Users/u-01HZX...

{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
  "id": "u-01HZX2N5...",
  "userName": "alice@acme.com",
  "name": { "givenName": "Alice", "familyName": "Lee" },
  "emails": [{ "value": "alice@acme.com", "type": "work", "primary": true }],
  "active": true,
  "groups": [{ "value": "engineering", "display": "Engineering" }],
  "meta": {
    "resourceType": "User",
    "created": "2026-05-20T14:32:11Z",
    "lastModified": "2026-05-20T14:32:11Z",
    "location": "https://identity-us.oyatie.com/scim/v2/Users/u-01HZX2N5..."
  }
}
```

---

## W. Sample SAML metadata exchange

### W.1 oyatie SP metadata (provided to Okta admin)

```xml
<?xml version="1.0"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata"
                     entityID="https://identity-us.oyatie.com/saml/sp/acme">
  <md:SPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <md:NameIDFormat>urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress</md:NameIDFormat>
    <md:AssertionConsumerService
        Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
        Location="https://identity-us.oyatie.com/saml/acs/acme"
        index="1"/>
    <md:SingleLogoutService
        Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect"
        Location="https://identity-us.oyatie.com/saml/slo/acme"/>
  </md:SPSSODescriptor>
</md:EntityDescriptor>
```

---

## X. Change log

- **2026-05-20** — Comprehensive rewrite (from 139-line stub to ≥1500-line PRD) as part of keystone-bundle 2026-05-20 foundational-doctrine documentation pass. Closes `feedback_autonomous_implementation_artifacts` gap: identity is a hero substrate and MUST be intern-buildable from the doc alone. Adds B2C personas + ≥40 stories + ≥6 UX flows + per-pack compliance + minor handling per ADR-0292 + comprehensive Cedar gate + step-up class semantics + threat model + sample Cedar policies + sample OIDC discovery + sample ID token claims + BC details + SCIM samples + SAML samples.
- **2026-05-18** — Initial stub publication (139 lines).

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is legacy/local-feedback provenance only after ADR-0515; protected merge authority is `oya-ci-required`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, while ArgoCD remains separately authorized CD evidence with cosign, tenant namespace, and audit-chain controls.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `identity` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `identity` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 4 module pin(s) across 4 context(s).
- Scaling input: `per_request` with cell placement `Tier-0` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
