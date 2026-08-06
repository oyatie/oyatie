---
id: ADR-0273
status: Superseded
date: 2026-05-20
owners:
  - axis-mail
  - council-deliverability
  - council-privacy
  - council-security
  - council-architecture
  - ops-sre-reliability
  - ops-deliverability
  - ops-secrets
  - axis-tenancy
  - axis-audit-chain
  - axis-policy-engine
  - axis-cloud-secrets
  - axis-cloud-network-dns
supersedes: []
amends:
  - ADR-0201-email-transactional-comms-adapter-substrate.md
superseded_by: [ADR-700]
related:
  - ADR-0008-secret-management-and-rotation.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0056-tenant-isolation-defense-in-depth.md
  - ADR-0064-canonical-base-plus-localization-pack.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0106-port-in-kernel-inward-only-flow.md
  - ADR-0117-regional-compliance-pack-data-residency.md
  - ADR-0123-event-bus-schema-registry.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-policy-governance-lane.md
  - ADR-0133-cellular-architecture-amazon-shape.md
  - ADR-0135-dual-context-isolation-kernel-invariant.md
  - ADR-0139-finops-tag-sustainability.md
  - ADR-0140-cedar-policy-engine.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0166-event-schema-versioning.md
  - ADR-0173-vendor-lock-in-avoidance.md
  - ADR-0174-finops-cost-attribution.md
  - ADR-0201-email-transactional-comms-adapter-substrate.md
  - ADR-0208-mail-microservice-substrate.md
  - ADR-0210-mail-jmap-imap-smtp-protocol-surface.md
  - ADR-0215-multi-context-platform.md
  - ADR-0238-openbao-secrets-storage.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
related_specs:
  - /specs/microservices/mail.json
  - /specs/per-microservice-flat-layout.json
  - /specs/dns-automation.json
  - /specs/secrets-rotation.json
  - /specs/event-schema-registry.json
related_prds:
  - microservices/mail/PRD.md
related_memory:
  - feedback_quality_performance_scalability_bar
  - feedback_autonomous_implementation_artifacts
  - feedback_no_silent_regression
  - feedback_canonical_base_localization
  - feedback_clean_architecture_requirements
  - feedback_bominal_inheritance_precedence
  - feedback_oya_git_canonical_2026_05_18
tags:
  - mail
  - deliverability
  - dkim
  - spf
  - dmarc
  - bimi
  - arc
  - multi-tenant
  - secrets-rotation
  - dns-automation
  - tier-1
  - lockdown
doc_class: Architecture-Decision-Record
tier: tier-1-lockdown
ship_blocker_for: [mail]
---

> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Per-tenant DKIM/SPF/DMARC — email deliverability ops

# ADR-0273 — Per-tenant DKIM/SPF/DMARC email deliverability

> Tier-1 lockdown. The `mail` µservice cannot ship — neither the
> B2C Personal Mail surface nor the B2B Work Mail surface — until
> the per-tenant DKIM/SPF/DMARC pipeline described here is built,
> wired into `cloud-secrets` / `cloud-network-dns` / `audit-chain`
> / `events-bus`, and observed end-to-end at SLO. This ADR is the
> deliverability contract for every outbound and inbound mail flow
> in oyatie, including `oyatie-corp` itself (per ADR-0242).

## Status

Proposed — gate for `mail` GA. Promotion to Accepted requires:

1. Working reference cell (`oyatie-corp` tenant) signing all
   outbound mail with both Ed25519 (RFC 8463) and RSA-2048
   (RFC 6376) DKIM selectors, passing `dkim=pass` at Gmail,
   Outlook.com, Yahoo, Apple iCloud, and Fastmail.
2. SPF flatten pipeline emitting a record under the 10 DNS
   lookup limit (RFC 7208 §4.6.4) for `oyatie-corp` and at
   least one synthetic stress tenant with 47 nested includes.
3. DMARC aggregate report (rua) ingestion live with at least
   72 hours of report data persisted in `audit-chain`,
   surfaced in the tenant deliverability dashboard.
4. BIMI selector resolving and rendering in at least Gmail
   and Apple Mail for a VMC-bound tenant logo.
5. ARC seal verified across a Google Groups → oyatie relay
   and an oyatie → external forward.
6. Gmail bulk-sender 0.3% complaint ceiling probe (see §D-9
   verification) green for two consecutive weeks.

Until all six gates land, the `mail` µservice ships under the
"closed-beta" certification level only (per ADR-0251) and may
not be enabled for any external tenant.

## Context

### Why this is tier-1 lockdown

Email is the single hardest substrate to ship correctly. A
mistake on inbound auth (SPF/DKIM/DMARC) silently routes
phishing into customer inboxes; a mistake on outbound auth
silently routes legitimate mail to recipient spam folders;
either failure is invisible until reputation has already
collapsed. Recovery from a single sustained complaint event
(>0.3% over Gmail's measurement window) requires weeks of
warm-up, IP rotation, and direct intervention with
Postmaster Tools — there is no quick rollback. We therefore
treat the deliverability stack as a ship-blocker, not a
follow-up.

Three external pressures collapsed onto the same date:

1. **Gmail Bulk Sender Requirements (Feb 2024).** Senders
   with more than 5,000 messages/day to Gmail recipients
   must (a) authenticate with both SPF *and* DKIM, (b)
   publish a DMARC record with at least `p=none` aligned
   to the From: header, (c) keep spam complaint rate
   below 0.3% (target <0.1%), (d) honor RFC 8058
   one-click unsubscribe with both `List-Unsubscribe`
   and `List-Unsubscribe-Post: List-Unsubscribe=One-Click`.
   Yahoo's parallel program imposes the same bar.
2. **Microsoft 365 outbound posture (May 2025).** Microsoft
   announced an analog of Gmail's bulk-sender rules for
   `outlook.com`, `hotmail.com`, `live.com`, and consumer
   `msn.com` traffic, with DMARC alignment enforcement and
   a `p=quarantine` minimum recommendation for senders
   exceeding 5k/day. The same DKIM-and-SPF-both rule
   applies.
3. **Apple Hide-My-Email (2024).** Apple's iCloud+ relay
   address pool routes through Apple-controlled DKIM
   signing. Outbound mail from oyatie to a Hide-My-Email
   alias must (a) pass DMARC alignment so the relay does
   not strip it, (b) not contain prohibited URL
   redirectors that trip Apple's tracker classifier, (c)
   honor `List-Unsubscribe` with both `mailto:` and
   `https://` so Apple Mail can show the one-tap
   unsubscribe button, (d) not depend on cookies or
   webhooks for unsubscribe state.

Any of the three platforms degrading our reputation
silently kills inbox placement across the rest of the
ecosystem because they publish reputation signals into
shared databases (Spamhaus SBL/XBL/PBL, Cloudmark CSI,
URIBL, SURBL) consumed by every downstream MTA.

### Why per-tenant, not per-cluster

ADR-0201 already scoped DKIM as per-tenant. We extend that
ruling: SPF, DMARC, BIMI, ARC, and reputation accounting
are also per-tenant. Reasons:

- A single cluster-wide DKIM key would cross-contaminate
  tenant reputation. One tenant's compromised key would
  blow up sender reputation for every other tenant on the
  same cluster.
- DMARC policy progression (`p=none → quarantine → reject`)
  must move at each tenant's own pace; some tenants come
  from legacy stacks with unauthenticated forwarders that
  need months to clean up.
- BIMI VMC certificates are issued to a specific legal
  entity tied to a specific From: domain. They are not
  shareable.
- Compliance overlays (KR-PIPA, EU GDPR processor, US
  HIPAA, US-Healthcare BAA, JP APPI, KSA NDMO) require
  tenant-bound key custody. A cluster-wide DKIM key would
  collapse the legal model.
- FinOps cost attribution (ADR-0174) requires per-tenant
  send metering, which is naturally aligned with
  per-tenant authentication keys.

### Scope boundary

In scope: outbound DKIM signing, outbound SPF record
emission, outbound DMARC publication and progression,
inbound DKIM/SPF/DMARC verification, ARC chain handling
on forward, BIMI selector emission, bounce + complaint
event normalization, RUA report ingestion, blocklist
monitoring, warm-up cadence, complaint-rate alarming.

Out of scope (deferred to dedicated ADRs):

- MTA-STS and TLS-RPT records (covered by a future
  inbound TLS posture ADR; the `mta-sts.txt` and
  TLS-RPT pipeline reuses the same DNS automation surface
  as this ADR).
- S/MIME and OpenPGP message-level encryption (a future
  ADR on end-to-end mail crypto).
- ActiveSync deliverability (M05 roadmap).
- SMS / push notifications (ADR-0201 explicitly defers).

## Decision

We adopt the twelve decisions D-1 through D-12 below. They
form the contract between the `mail` µservice, the
`cloud-secrets` and `cloud-network-dns` µservices, the
`audit-chain` and `events-bus` substrates, and the per-tenant
control plane. Every decision is mandatory; any partial
deployment fails the §Verification gates.

The implementation surface (crates, helm charts, runbooks,
specs) is enumerated in §"Implementation surface". The full
test matrix is enumerated in §Verification.

---

### D-1 — Per-tenant custom-domain DKIM keys (Ed25519 + RSA-2048 dual)

Every tenant that sends mail under a custom From: domain
gets its own pair of DKIM signing keys:

- **Ed25519** key (RFC 8463), selector `oya-ed-<rotation_epoch>`.
- **RSA-2048** key (RFC 6376 §3.3), selector `oya-rsa-<rotation_epoch>`.

Both keys are present on every outbound message under that
From: domain. Verifiers that support Ed25519 (Gmail,
Outlook, Yahoo, Apple Mail, Fastmail, Proton as of 2025)
will read the Ed25519 signature; legacy MTAs that only
support RSA fall back to the RSA-2048 signature without
failing DKIM.

The dual-algorithm posture is a hedge: Ed25519 signatures
are smaller and constant-time-verifiable but not universally
supported by 2026. RSA-2048 is universally supported but
slated for deprecation against RSA-3072 some time after
2030 (per NIST SP 800-131A guidance). When NIST moves us
off RSA-2048, the dual-selector model lets us swap in
RSA-3072 (or post-quantum DKIM, if that lands) without an
outage. The keys are *not* dual-signed in the sense of
parallel cryptographic chains; they are two independent
`DKIM-Signature:` headers per message, each verifiable in
isolation.

#### Storage

Keys are stored in the `cloud-secrets` µservice
(OpenBao-backed per ADR-0238). Each tenant gets a secret
path of the form:

```
secret/data/tenants/<tenant_id>/mail/dkim/<domain>/<algorithm>/<selector>
```

Stored fields:

- `private_key_pem` (Ed25519 PKCS#8 PEM or RSA PKCS#1 PEM)
- `public_key_b64` (DNS-ready base64 of the SubjectPublicKeyInfo)
- `selector` (e.g. `oya-ed-2026q2-001`)
- `created_at`, `not_before`, `not_after`
- `algorithm` (`ed25519` or `rsa-2048`)
- `key_id` (uuid v7, monotonic)
- `dns_published_at` (timestamp set by the DNS publisher when
  the TXT record is observed propagated to all authoritative
  NS for the zone)
- `dns_observed_propagated_at` (timestamp set by the resolver
  probe when at least N=8 public resolvers return the
  record; until this is set, the signing pool refuses to
  use the key)
- `revoked_at` (nullable; set on emergency rotation)

Access is gated by Cedar (ADR-0140 / ADR-0243): only the
`mail::SigningPool` principal in the same tenant scope may
read the private key. Audit-chain (ADR-0028) records every
read event. No human principal is permitted to read a
private DKIM key. Break-glass procedures (ADR-0241) require
council-security four-eyes and forcibly trigger immediate
rotation (D-2) on use.

#### key-custody-BYOK

Tenants under ADR-0251 key-custody-BYOK posture may bring their own
DKIM keys generated in their own KMS region. The
`cloud-secrets` adapter for key-custody-BYOK tenants wraps the
tenant-KMS-resident key behind the same secret-path API; the
signing pool sees the same interface but the actual private
operations happen in the customer KMS. Ed25519 + RSA-2048
dual is preserved.

#### Key material format and provenance

Key generation uses the platform's audited cryptographic
library (the `ring` crate for Ed25519, the same library
backing rustls). The generation event is recorded in
audit-chain with the public key fingerprint, the tenant
id, the rotation epoch, and the requesting workflow id.
This makes every signing key traceable to a generation
event reachable from the audit chain Merkle root.

#### DNS publication

The public half of each key is emitted as a TXT record at:

```
<selector>._domainkey.<domain>.   IN TXT  "v=DKIM1; k=ed25519; p=<base64>"
<selector>._domainkey.<domain>.   IN TXT  "v=DKIM1; k=rsa; p=<base64>"
```

Publication is performed by the `cloud-network-dns`
µservice, which in turn delegates to the per-tenant
authoritative DNS provider (Route 53, Cloudflare DNS,
NSOne, in-house BIND, depending on the tenant's choice
under ADR-0240). The DKIM key is *not* usable for signing
until D-1 verifies propagation across at least eight
public resolvers (Google 8.8.8.8 / 8.8.4.4, Cloudflare
1.1.1.1 / 1.0.0.1, Quad9 9.9.9.9, OpenDNS 208.67.222.222,
Yandex 77.88.8.8, AdGuard 94.140.14.14) and the tenant's
own authoritative NS set returns NOERROR with the expected
public key payload.

---

### D-2 — Automated DKIM key rotation every 90 days

Every DKIM key — Ed25519 and RSA — rotates on a 90-day
cadence with overlap windows on both ends to prevent
signature gaps. The rotation state machine has five states:

1. `PENDING` — key generated, not yet published to DNS.
2. `PUBLISHED` — DNS TXT record live but not yet
   propagation-verified.
3. `ACTIVE` — propagation verified, key is the primary
   signing selector for new outbound mail. There is
   always exactly one `ACTIVE` Ed25519 key and one
   `ACTIVE` RSA key per tenant per From: domain.
4. `DEPRECATED` — superseded by a newer `ACTIVE` key; no
   new outbound mail signs with this selector, but the
   DNS record stays published for 7 days (the
   `deprecation overlap`) so receivers can still verify
   in-flight mail.
5. `RETIRED` — DNS record removed; private key purged
   from `cloud-secrets`; the public key fingerprint
   stays in the audit chain forever.

Cadence:

- T-7d: workflow generates a new `PENDING` key, asks
  `cloud-network-dns` to publish, polls until
  `PUBLISHED → ACTIVE`. If propagation never resolves
  within 24 hours the workflow alerts and aborts; the
  old `ACTIVE` key keeps running, no outage.
- T+0: new key flips to `ACTIVE`; old key flips to
  `DEPRECATED`.
- T+7d: deprecation overlap ends; old key flips to
  `RETIRED`; DNS record removed; secret purged.

The 7-day overlap is calibrated for the worst observed
"slow forwarder" tail in production data (Gmail's longest
deliberate forwarding queues hit ~6 days in 2024 incident
post-mortems; we add a day for safety).

#### Rotation triggers

Beyond the 90-day cadence, rotation is also triggered by:

- Manual revocation (council-security four-eyes).
- Audit-chain detected anomaly (e.g., a signing operation
  from outside the expected tenant cell, possibly
  indicating key exfiltration).
- A `dkim=fail` rate over 1% sustained for 1 hour against
  the active selector (probable selector misconfiguration).
- A `cloud-secrets` rekey event (e.g., OpenBao master
  rotation forces a downstream rewrap that we use as a
  rotation opportunity).

#### Rotation runbook

The runbook lives at
`microservices/mail/runbooks/dkim-rotation.md`. It is
executable by the Foundry pipeline (per
ADR-0116 / ADR-0145) as a self-driving workflow. Human
operators have no manual DKIM-key path; emergency manual
rotation goes through the break-glass workflow which
itself drives the same state machine.

#### Why 90 days

- Longer than 30 (avoids DNS churn pressure on
  authoritative providers).
- Shorter than 365 (NIST SP 800-57 part 1 §5.6.4
  "cryptoperiod for digital signature private keys"
  recommends 1-3 years for most uses but the deliverability
  community converges on quarterly DKIM rotation for
  reputation hygiene).
- Aligns with TLS certificate rotation cadence (LetsEncrypt
  90d) so operations cognitive load is uniform.
- Matches the SOC2 control "cryptographic keys are
  rotated at defined intervals" with a defensible
  interval.

---

### D-3 — SPF flatten + nested record management

SPF (RFC 7208) is brittle: every `include:` directive
consumes one of the 10 DNS lookups allowed per evaluation
(RFC 7208 §4.6.4). Tenants who use third-party senders
(SES, Mailgun, SendGrid, Postmark, Zendesk, Salesforce
Marketing Cloud, Marketo, HubSpot, Intercom, ZoomInfo,
Workday, Greenhouse, etc.) blow the limit within four or
five integrations.

#### Flattening pipeline

The `mail` µservice runs a per-tenant SPF flattening
workflow that:

1. Reads the tenant's declared sender providers from the
   tenant's mail configuration document
   (`tenant_config/mail/spf_authorized_senders`).
2. Resolves each provider's published SPF record by
   recursive `include:` expansion to fully-qualified
   `ip4:` / `ip6:` mechanisms.
3. Coalesces overlapping ranges and emits a single
   flattened TXT record at the From: domain.
4. Publishes through `cloud-network-dns`.
5. Re-runs daily (a cron workflow) and on any
   provider-side IP-pool change observed via the
   provider's own RSS / API feed; emits a diff to
   audit-chain.
6. Falls back to the unflattened `include:` form if
   flattening would exceed 450 octets in a single TXT
   string (the safe DNS message size before TCP
   fallback) or 10 mechanism count after coalescing.

Flattening trades freshness (a provider rotating their
IPs sees us update within 24h, not instantly) for
correctness (we never blow the lookup limit and tank
every recipient's SPF check). The trade is reviewed daily
in the deliverability dashboard.

#### Static fallback record

Every tenant publishes, in addition to the dynamic
flattened record, a `_spf-fallback.<domain>` TXT record
that contains the last known-good unflattened form. If
the flattener job is broken for more than 72h, the
override switch in `cloud-network-dns` allows ops to swap
the live record back to the fallback. The fallback is
generated by the same workflow; ops cannot hand-edit it.

#### Nested record management

When tenants have subdomain sending (`marketing.acme.com`,
`receipts.acme.com`, `notifications.acme.com`) each
subdomain has its own SPF record managed separately. The
parent domain's record covers transactional mail; each
marketing subdomain's record covers its own ESP. This
prevents marketing-provider IP churn from invalidating
transactional SPF.

#### Policy mechanism

We default the trailing mechanism to `~all` (softfail)
during warm-up and progress to `-all` (hardfail) once
DMARC enforcement reaches `p=reject` (see D-4). Tenants
may not stay on `?all` (neutral) or `+all` (pass all)
past the warm-up window; the policy validator refuses
those forms in `ACTIVE` state and the deliverability
dashboard flags them as a tier-1 finding.

---

### D-4 — DMARC policy progression (`p=none → p=quarantine → p=reject`) per-tenant

Every tenant publishes a DMARC record (RFC 7489) at
`_dmarc.<domain>`. The policy advances through three
gates:

- **`p=none`** (observation). Default for new tenants
  during onboarding. Receivers honor SPF and DKIM
  separately; no action taken on alignment failure. RUA
  reports flow in.
- **`p=quarantine`** (junk-folder enforcement).
  Receivers route failing mail to spam. Reached after 14
  consecutive days of `p=none` with `pct=100`, less than
  0.5% alignment failure observed in inbound RUA, and at
  least 1000 messages in the observation window.
- **`p=reject`** (hard bounce). Receivers reject failing
  mail at SMTP time. Reached after another 14
  consecutive days of `p=quarantine` with less than 0.1%
  alignment failure observed.

Progression is workflow-driven, not human-driven. A
human override (council-deliverability four-eyes) can
hold a tenant at `p=quarantine` indefinitely if their
legacy mail flow has unfixable alignment problems (e.g.,
a vendor that forwards from a non-aligned domain). The
override is recorded in audit-chain and the dashboard
flags it as a "stuck at quarantine" exception.

#### Subdomain policy

The `sp=` tag is set explicitly. Default: same as `p=`.
Tenants who delegate marketing to subdomains may set
`sp=reject` once the parent reaches reject; the
flattener + DKIM publisher must be live on the
subdomain first.

#### Reporting tag set

Every DMARC record carries:

- `rua=mailto:dmarc-rua@reports.<oya-tenant>.oyamail.io`
  — aggregate report endpoint hosted on the
  `mail-reports` cell. Hostname format is fixed so the
  ingestion router doesn't need per-tenant
  configuration.
- `ruf=mailto:dmarc-ruf@reports.<oya-tenant>.oyamail.io`
  — forensic (failure) report endpoint; tenants may
  opt out of `ruf` per ADR-0140 privacy gating, since
  forensic reports can contain message content of
  third-party complainants.
- `fo=1` — generate forensic report on any underlying
  authentication failure (not just DMARC alignment
  failure).
- `adkim=s` (strict DKIM alignment) and `aspf=s`
  (strict SPF alignment) for tenants past `p=quarantine`.
- `pct=100` always in `p=quarantine` and `p=reject`;
  `pct=` is not used to softly enforce — we either
  enforce or we don't.

The hostname `oyamail.io` is a separate operational
domain from the tenant's mail domain so that even if a
tenant's domain is locked out of DNS we can still
ingest reports. It is a substrate-owned domain (per
ADR-0245).

---

### D-5 — DMARC aggregate report (rua) processing + per-tenant dashboard

The `mail-reports` cell exposes the
`dmarc-rua@reports.<tenant>.oyamail.io` mailbox. Reports
arrive as ZIP/GZIP attachments on standard scheduled
(daily) emails from receivers. The pipeline:

1. SMTP receiver on the `mail-reports` cell accepts mail
   to `dmarc-rua@*` and writes the raw eml to
   tenant-scoped object storage (per ADR-0117 residency
   pack).
2. A worker decompresses the attachment, validates the
   XML schema against the IETF DMARC RUA schema (the
   2023 IETF draft schema; we follow the IETF stream
   because it tracks the upstream working group), and
   normalizes each record into the canonical
   `dmarc.aggregate.v1` event in the schema registry
   (per ADR-0166).
3. The event is published on `events-bus` and consumed
   by the `audit-chain` ingester and the
   `mail-deliverability-dashboard` projector.
4. Pathological records (XML parse failure, schema
   mismatch, oversized > 64 MiB) are quarantined for
   manual triage. Quarantine count is itself an SLO
   indicator.

#### Per-tenant dashboard

The deliverability dashboard exposes, per tenant per
From: domain per day:

- Total volume reported.
- Pass rate (DMARC, DKIM-only, SPF-only).
- Top source IPs by volume.
- Source IPs that fail DMARC alignment (with reverse-DNS,
  ASN, geolocation).
- Alignment failure breakdown by failure cause (SPF
  domain mismatch, DKIM domain mismatch, both).
- Forwarder detection (ARC-Authentication-Results: matches
  a known forwarder pattern → reports do not block
  progression).
- Suggested next-action banner ("you are eligible to
  progress to p=quarantine — apply").

The dashboard is read-only for tenant admins. Apply /
revert actions go through the workflow described in D-4.

#### Retention

Aggregate reports retain 18 months by default (longer
than Google Postmaster Tools' own 60-day window, so we
can answer slow-moving forensic questions). Tenants on
HIPAA / legal-hold may extend retention up to the
tenant's mailbox retention floor.

---

### D-6 — BIMI deployment for verified branding

BIMI (RFC 9695, finalized 2024) lets a tenant publish a
logo that recipient MUAs (Gmail, Apple Mail, Yahoo
Mail, Fastmail) render in the From: column once DMARC
enforcement is at `p=quarantine` or `p=reject` and a
Verified Mark Certificate is bound.

#### Bring-your-own VMC

Tenants procure their own VMC from a BIMI-authorized CA
(Entrust, DigiCert, GlobalSign as of 2026). The
`mail` µservice does *not* broker VMC issuance — it is
a trademark-attestation process the tenant's legal team
owns. The µservice does:

- Validate the SVG logo against the BIMI Tiny Profile
  (SVG 1.2 Tiny PS, base shape primitives only, no
  scripts, no external references, <32 KiB after
  gzip).
- Validate the VMC against the BIMI VMC issuance
  policy (chain to a BIMI-authorized root, EV
  organization match, logotype extension present).
- Publish the BIMI selector record at
  `<selector>._bimi.<domain>.   IN TXT
  "v=BIMI1; l=<svg-url>; a=<vmc-url>"`.
- Cache the SVG and VMC at substrate-owned CDN
  endpoints so the recipient MUA fetches them from
  edge nodes with predictable latency.

#### Selector strategy

Default selector is `default`. Tenants may publish per
brand selectors (`marketing._bimi`, `support._bimi`)
matching subdomain From: addresses; each requires its
own VMC.

#### Gate

BIMI publication is gated on DMARC `p=quarantine` (per
RFC 9695 §8.1). The DMARC progression workflow (D-4)
emits an event when a tenant crosses the threshold; the
BIMI publisher consumes that event and unlocks the BIMI
opt-in flow in the tenant admin console. Until the
threshold is crossed, the BIMI publisher refuses to
publish the selector record (publishing a BIMI record
without DMARC enforcement is a no-op that confuses
auditors).

#### CMC (Common Mark Certificate)

Tenants without a registered trademark may use a CMC
(prior-use mark) once BIMI authorities accept it. As
of 2026 only Entrust and DigiCert issue CMCs; the
µservice supports both certificate types transparently.

---

### D-7 — ARC signing for forwarded mail

ARC (RFC 8617) preserves authentication results across
relays. Two ARC flows matter:

#### Inbound ARC verification

When mail reaches the `mail` µservice through a forwarder
(e.g., a Google Group, a mailing list, a tenant's own
legacy MX rerouting to oyatie), the original
authentication may have failed but ARC headers may attest
that the previous hop passed. The inbound verifier:

1. Walks the ARC-Authentication-Results chain in
   reverse, validating each ARC seal's signature.
2. If a chain link is valid up to an ARC-trusted
   forwarder (the trusted-forwarder list lives at
   `microservices/mail/policy/arc-trusted-forwarders.yaml`
   and is reviewed quarterly), the message's effective
   DMARC alignment uses the original
   ARC-Authentication-Results, not the broken
   post-forward auth.
3. The event includes `arc=pass via <forwarder>` so the
   dashboard can attribute pass-through traffic
   correctly and DMARC progression (D-4) does not
   over-count alignment failures caused by ARC-able
   forwarders.

#### Outbound ARC signing

When a tenant uses oyatie's mailing-list / shared-mailbox
/ forwarding features (Work Mail group mailbox, Personal
Mail vacation-responder forwarding, server-side Sieve
rules with redirect), we ARC-sign the message on egress
so the downstream verifier can chain auth back to us
even though the From: header is the original sender's.
The ARC seal uses the same Ed25519 + RSA-2048 dual
selectors as DKIM (D-1), with selector prefix
`oya-arc-` to keep them visually distinct.

#### Trust anchor

oyatie itself joins the public ARC-trust expansion
(M3AAWG ARC trust signal exchange) once we have 90 days
of low-complaint outbound history per tenant. The
`oyatie-corp` tenant joins first (per ADR-0242
dogfooding) and serves as the test bed for trust
signalling.

---

### D-8 — Outbound deliverability monitoring + complaint tracking

Outbound mail emits seven event kinds on `events-bus`
(matching ADR-0201's schema, with extensions in this
ADR):

- `sent` — message left our MTA.
- `delivered` — recipient MTA accepted at 2xx.
- `deferred` — recipient MTA gave 4xx (transient).
- `bounced` — recipient MTA gave 5xx (hard).
- `complained` — recipient marked as spam (via
  feedback-loop subscription).
- `blocked` — recipient MTA refused before message body
  (RBL hit, IP block).
- `unsubscribed` — `List-Unsubscribe` triggered or
  feedback-loop `unsubscribe`.

Every event carries (a) the tenant id, (b) the From:
domain, (c) the recipient MX vendor classification
(`gmail`, `m365`, `yahoo`, `apple_icloud`, `proton`,
`fastmail`, `chinese-isp`, `other`), and (d) the
sender-IP-pool id.

#### Per-tenant SLO

- Complaint rate per recipient MX, rolling 24h:
  warn at >0.05%, page at >0.1%, lockout new sends
  at >0.3% (per Gmail bulk-sender threshold).
- Bounce rate, rolling 24h: warn at >2%, page at >5%,
  pause sending at >10%.
- DKIM signing success rate: warn at <99.99%, page at
  <99.9%. (Any DKIM failure is treated as a substrate
  bug, not a tenant problem.)
- DMARC alignment rate (from RUA): warn at <99%, page
  at <97%.

Lockouts route through the tenant admin console with a
"deliverability hold" banner. The tenant can
acknowledge and request a warm-up restart (D-10);
they cannot disable the alarm.

#### Sub-IP-pool reputation

Tenants may opt into a dedicated outbound IP pool (per
ADR-0201). The reputation telemetry partitions by pool;
a noisy tenant inside a shared pool is moved into a
penalty pool within 6h of crossing the complaint warn
threshold, isolating the rest of the shared-pool
tenants. The penalty pool routes through a slower
egress profile (max 10 msg/sec, low-priority queue)
until reputation recovers.

---

### D-9 — Bounce + complaint handling (Apple Hide-My-Email + Gmail 2024 bulk requirements)

Gmail's 2024 bulk-sender requirements and Apple's
Hide-My-Email policies impose three concrete obligations
beyond what ADR-0201 already covers:

#### One-click unsubscribe

Bulk marketing mail (defined as messages where
`X-Oyatie-Mail-Class: marketing` or where the tenant
sends to >5000 distinct recipients in 24h) must
include:

- `List-Unsubscribe: <https://unsub.<tenant>.oyamail.io/u/<opaque-token>>, <mailto:unsub+<opaque-token>@reports.<tenant>.oyamail.io>`
- `List-Unsubscribe-Post: List-Unsubscribe=One-Click`

The HTTPS endpoint must complete unsubscribe with a
single POST (RFC 8058). The opaque token encodes
tenant id + recipient id + campaign id + an HMAC over
those three values, so it cannot be forged.
Unsubscribe state propagates to the tenant
suppression list within 60 seconds.

The mailto: form must process unsubscribe within 24h
(RFC 8058 hard ceiling) but we target 60 seconds for
consistency with the HTTPS form. The mailto:
processor is the same `mail-reports` cell as DMARC
RUA, sharing the SMTP receiver.

#### Complaint feedback loop subscriptions

We subscribe to every major feedback loop:

- AOL/Yahoo FBL (single feed for both since the 2017
  merger).
- Microsoft JMRP and SNDS.
- Comcast FBL, Cox FBL, USA.net FBL.
- Google does *not* offer a public FBL; we use
  Google Postmaster Tools API for reputation, complaint
  rate, and authentication telemetry instead.
- Apple does *not* offer a public FBL; we infer
  complaints from Hide-My-Email auto-deactivation
  signals (a Hide-My-Email alias that suddenly returns
  5xx after a series of 2xx is a complaint signal).

All FBL inputs flow into the `complained` event stream
(D-8).

#### Apple Hide-My-Email specifics

- We never use the recipient's Hide-My-Email alias
  as a primary key inside the tenant CRM. We treat it
  as an ephemeral routing address; the canonical
  recipient identity is the alias-resolved user id
  (if the tenant has user-account context) or a hash
  of the alias as observed. This avoids accidentally
  pinning a tenant's CRM to an alias the recipient
  will rotate.
- We honor Apple's "Mail Privacy Protection" by *not*
  treating tracking-pixel opens from Apple Mail user
  agents as evidence of engagement. The tracking-pixel
  classifier in `mail-analytics` excludes Apple Mail
  user agents from open-rate computation entirely.
- We render `List-Unsubscribe` in a form Apple Mail
  parses (both URL and mailto:); we do not require
  cookies or JavaScript on the unsubscribe page.

#### Complaint threshold gates (Gmail 0.3%)

The complaint-rate gate at the tenant level is a
hard rule:

- Sustained over rolling 24h on Gmail recipients
  specifically.
- >0.3% → outbound to Gmail-class recipients pauses
  immediately. The tenant admin sees a banner: "Gmail
  complaint rate exceeded — your outbound to Gmail
  recipients is paused for 6 hours. Investigate
  causes in the deliverability dashboard. Resume
  requires acknowledgment + a warm-up restart."
- >0.5% → outbound to all bulk-sender-policy
  recipients (Gmail + Yahoo + Microsoft + Apple)
  pauses for 24h.
- >1% → tenant is moved into the penalty IP pool
  (D-8) and outbound is throttled to 10 msg/sec for
  72h regardless of recipient.

We deliberately treat these as automation, not human
review, because manual review at 0.3% complaint rate
is already too slow — the damage to sender reputation
happens within an hour.

---

### D-10 — Per-tenant warm-up cadence for new sending domains

A new From: domain starts cold: no reputation, no
warm-up history. Cold domains that send at full volume
on day one get classified as spam by every major
receiver within hours. The warm-up cadence:

- Day 0–1: 50 messages/day to known-good seed list (the
  tenant's own employee addresses + opt-in beta cohort).
- Day 2–3: 200 messages/day to seed list + first 5% of
  the tenant's real mailing list.
- Day 4–7: 1000 messages/day, expanding to first 25% of
  list.
- Day 8–14: 5000 messages/day, expanding to 100% of
  list, splitting evenly across Gmail / Microsoft /
  Yahoo / Apple bucket.
- Day 15–30: linear ramp to 50000 messages/day if
  complaint and bounce rates stay below half the SLO
  warn threshold (D-8).
- Day 31+: full volume.

The cadence is enforced at the MTA queue. A tenant who
tries to blast a 200k list on day 3 sees the excess
queued with a delivery-deferred event and a banner on
the dashboard: "warm-up cadence active, your message
will queue across the next N days".

#### IP-pool warm-up

The same cadence applies to a new outbound IP
(dedicated-pool tenants or after a substrate-level
IP-pool replacement). IPs warm independently of
domains, so a tenant moving to a dedicated pool resets
the IP clock but keeps the domain clock.

#### Cooldown

A tenant whose complaint rate spikes (D-9) is moved
back through the warm-up ladder: from full volume to
day-15-equivalent, then a 7-day ramp to full again.
This prevents "fix the cause and immediately blast
again" anti-patterns.

#### Manual override

The council-deliverability four-eyes override (per
D-4) can shorten the cadence in exceptional cases
(e.g., a tenant migrating an established mailing list
from another reputable provider with provable
warm-up history at the previous provider). The
override is logged and audited.

---

### D-11 — Blocklist monitoring + remediation workflow

We monitor tenant From: domains and outbound IP pools
against a curated blocklist set:

- Spamhaus (SBL, XBL, PBL, DBL, ZEN, ROKSO).
- SURBL multi.
- URIBL black + grey + multi.
- Invaluement (IVDNSBL).
- Barracuda.
- SpamCop.
- SORBS.
- Apple iCloud's private blocklist (inferred from
  bounce-code patterns: 5.7.1 bounces with "Message
  rejected due to local policy").
- Microsoft Smart Network Data Services (SNDS).
- Google Postmaster Tools reputation feed.

Probes run every 10 minutes from substrate-owned
resolvers (with diverse network egress points so a
single ISP block doesn't blind the probe). A hit
generates a `blocklist.hit.v1` event on `events-bus`,
which the deliverability workflow consumes.

#### Remediation workflow

On a confirmed hit:

1. Page the on-call deliverability operator.
2. Auto-collect evidence: the blocklist's listing
   reason, the last 24h of outbound from the affected
   IP/domain, the complaint and bounce rates.
3. Open a remediation ticket in the tenant's audit
   queue and in the substrate ops queue. Both queues
   reference the same evidence bundle (audit-chain
   shared anchor).
4. Run the blocklist's own delisting procedure
   (Spamhaus has an API; the rest are mostly web
   forms or email-based; the workflow tracks each).
5. Throttle outbound to bulk recipients to 10 msg/sec
   from the affected IP until delisted.
6. After delisting, re-enter the cooldown ramp from
   D-10.

The remediation workflow is *not* one-click; it
requires evidence review by the on-call operator
before delisting requests are submitted, because false
positives on listing reason will burn substrate-level
relationship with the blocklist operator.

---

### D-12 — Inbound DKIM/SPF/DMARC verification + spam scoring

Every inbound message hitting the `mail` µservice runs
through the verification pipeline:

1. **SPF** (RFC 7208) — check the connecting IP
   against the From: domain's SPF record. Emit
   `spf=pass|fail|softfail|neutral|none|permerror|temperror`.
2. **DKIM** (RFC 6376) — verify every
   `DKIM-Signature:` header. If any signature
   verifies and aligns with the From: domain, emit
   `dkim=pass`. Otherwise emit `dkim=fail` or
   `dkim=none`.
3. **DMARC** (RFC 7489) — apply the From: domain's
   DMARC policy with alignment. Emit
   `dmarc=pass|fail` and `dmarc.policy=<published>` and
   `dmarc.effective_action=<our action>`.
4. **ARC** (RFC 8617) — if SPF/DKIM/DMARC fail, walk
   the ARC chain. If a trusted forwarder asserts an
   earlier pass, emit `arc=pass via <forwarder>` and
   accept the message with a downgraded auth score.
5. **MTA-STS / TLS-RPT** — if the connecting MTA
   honors MTA-STS for our inbound domain, record the
   compliance; if not, log under TLS-RPT. (Detailed
   policy in a future inbound TLS posture ADR.)

The resulting Authentication-Results: header is
written exactly once and signed with our inbound
DKIM key so internal stages can trust it.

#### Spam scoring

After auth, Rspamd runs:

- Per-tenant Bayesian classifier (the `intelligence`
  substrate, per ADR-0255, hosts the per-tenant
  classifier model).
- URL reputation (URIBL, SURBL, our own URL
  classifier).
- Sender reputation (per-IP, per-domain, per-ASN,
  blended).
- Content rules (custom per tenant, e.g., DLP rules
  flagging exfiltration patterns).
- Cryptographic anomaly rules (e.g., DKIM signed by a
  domain that just appeared in DNS in the last 24h is
  flagged for review).

The spam score combines with the auth result into a
final disposition:

- `auth=pass + spam_score<5` → inbox.
- `auth=pass + spam_score 5–9` → flagged in inbox
  with "this sender looks suspicious" warning.
- `auth=pass + spam_score>=10` → spam folder.
- `auth=softfail + any score` → spam folder unless
  user has whitelisted sender.
- `auth=fail` → rejected at SMTP-time (5.7.1) with a
  tenant-specific bounce message giving the
  authentication reason.

The `auth=fail` rejection is the default behavior; a
tenant may relax it (rare) to `quarantine instead of
reject` via Cedar policy, but the council-deliverability
override is required to do so on a regulated tenant
(HIPAA / FedRAMP).

---

## Alternatives considered

### Alternative A — Cluster-wide DKIM key

Use one DKIM key per cluster, with the public key
published on a substrate-owned domain
(`dkim.oyamail.io`) that every tenant CNAMEs into. Cuts
operational complexity by 100x.

Rejected because:

- Cross-contamination risk: any single tenant's
  compromised key blows reputation for every tenant on
  the cluster. Linus-style: "we do not ship that."
- Legal-entity binding: BIMI VMC requires the key
  custody to bind to the tenant's trademark holder.
  Shared keys break the BIMI legal model.
- key-custody-BYOK customers (per ADR-0251) demand tenant-bound
  key custody as a contract clause.
- Forensics: a phishing investigation that traces a
  signature back to "the cluster key" cannot localize
  which tenant's pipeline was involved.

### Alternative B — RSA-only DKIM (no Ed25519)

Stay on RSA-2048 only, on the grounds that universal
support is more important than smaller signatures.

Rejected because:

- Verifier support for Ed25519 (RFC 8463) is widespread
  as of 2025 across the recipients that matter for
  reputation (Gmail, Yahoo, Outlook, Apple, Fastmail,
  Proton). The remaining holdouts (small ISPs, legacy
  Exchange) fall back to RSA gracefully.
- Constant-time verification with Ed25519 reduces
  inbound CPU cost at scale.
- Future-proofs against the RSA-2048 deprecation
  glide path. We want the rotation infrastructure to
  exercise dual-selectors *now*, not when we're
  forced to add Ed25519 in a hurry.

### Alternative C — DMARC `p=reject` from day one

Skip the progression ladder and start every new
tenant at `p=reject`. Maximally protective.

Rejected because:

- New tenants invariably have unaligned legacy
  flows (a vendor sending receipts from an
  unauthenticated domain, a marketing tool whose
  SPF includes haven't been added yet). Day-one
  reject hard-fails those flows and ships customer
  pain.
- The `p=none → quarantine → reject` ladder is the
  industry-standard progression because it lets the
  RUA reports surface unauthenticated senders
  before they get hard-rejected.
- Receivers do not penalize a tenant for staying on
  `p=none` for the first 14 days; they do penalize
  a tenant who repeatedly flips between `reject`
  and `none`.

### Alternative D — Outsource the whole stack to SendGrid / Postmark / Mailgun

Don't build the DKIM/SPF/DMARC pipeline at all;
ship transactional through a SaaS, marketing
through a separate SaaS, and let those providers
own the deliverability problem.

Rejected because:

- Violates ADR-0173 vendor lock-in avoidance for a
  hero product.
- Forecloses sovereign-cloud and air-gapped
  deployments (ADR-0240) — SendGrid is not
  available in regulated jurisdictions on
  customer-controlled infra.
- key-custody-BYOK (ADR-0251) — SaaS providers don't expose
  their DKIM key custody to the customer.
- Compliance overlay packs (KR-PIPA, HIPAA BAA, KSA
  NDMO) require key custody that mainstream ESPs do
  not provide.
- The economics break at scale: per-message ESP
  pricing is multiple orders of magnitude over our
  cost-of-goods at hyperscaler-grade volume.

### Alternative E — Defer BIMI to post-GA

BIMI is "nice-to-have"; ship without it.

Rejected because:

- BIMI is now the visible-trust signal in Gmail and
  Apple Mail. Tenants whose competitors have BIMI
  but they don't visibly look "less legitimate" in
  user inboxes.
- BIMI's gating on DMARC `p=quarantine` aligns
  exactly with our other progression infrastructure
  — building it later means a second pass through
  the same code paths.
- VMC procurement is *tenant*-side work and can
  proceed in parallel with our build. We need only
  the publisher pipeline ready at GA.

### Alternative F — Skip ARC, accept forwarder breakage

Don't sign or verify ARC; treat forwarder failures
as user error.

Rejected because:

- Google Groups, Apple iCloud forwarding,
  Microsoft 365 distribution lists, and university
  mailing lists are pervasive in B2B traffic.
  Without ARC handling we lose ~3% of legitimate
  inbound to DMARC alignment failure (measured on
  pilot data from sibling deployments).
- M3AAWG ARC trust signal exchange is the path
  toward reduced false-positives across the
  industry; opting out forecloses participation.
- The outbound ARC seal is mandatory for our own
  group-mailbox feature; we can't ship Work Mail
  shared mailboxes correctly without it.

### Alternative G — Trust-but-verify human DKIM key access

Allow on-call SRE to view DKIM private keys for
debug purposes, gated by audit logging.

Rejected because:

- A private DKIM key in human RAM is a key that
  can be exfiltrated by phishing or insider risk.
  The audit log will tell you about the breach;
  it won't prevent it.
- "Audit it and move on" is the kind of
  short-term operational shortcut that ages
  badly. We will not build a substrate with a
  human-readable DKIM private key.
- The Foundry pipeline can drive every legitimate
  debug workflow (re-sign a test message, dump
  the public key, compare against DNS, etc.)
  without ever exposing the private key.

---

## Consequences

### Positive

- Per-tenant deliverability isolation; one tenant's
  catastrophe does not poison the rest of the cluster.
- key-custody-BYOK (ADR-0251) becomes a coherent extension of the
  existing DKIM custody model.
- The DMARC progression ladder produces measurable
  reputation, observable from day one, with clear
  graduation criteria.
- Aggregate-report ingestion gives tenants the kind of
  deliverability dashboard previously reserved for
  enterprise customers of Salesforce Marketing Cloud
  or SparkPost Signals.
- BIMI from day one lets tenants ship verified-brand
  mail as soon as they cross the DMARC threshold.
- ARC handling closes the forwarder-loss gap that
  routinely costs 1-3% of legitimate inbound traffic
  in vanilla DMARC enforcement.
- The 90-day rotation cadence + dual-algorithm dance
  prepares us for the eventual RSA-2048 retirement
  without a rushed migration.
- The complaint-rate automation enforces the Gmail
  bulk-sender bar without depending on human review.

### Negative

- Operational complexity: per-tenant DKIM keys mean
  per-tenant DNS records, per-tenant secret paths,
  per-tenant rotation workflows. Scales with tenant
  count; mitigated by full automation but adds load
  to `cloud-secrets` and `cloud-network-dns`.
- DNS authoritative-provider dependency: a DNS
  provider outage halts new key rollouts. Mitigated
  by the 7-day overlap window (D-2) and by
  multi-provider redundancy in `cloud-network-dns`.
- BIMI VMC procurement is on the tenant; some
  tenants will be confused or slow. Mitigated by
  dashboard guidance and CMC fallback.
- Warm-up cadence (D-10) is unpopular with tenants
  used to "click send, mail goes out". Mitigated by
  the dashboard explanation and by accepting the
  pain as a substrate principle: short-term tenant
  friction in exchange for long-term reputation.
- Storage cost: 18 months of DMARC RUA per tenant
  per day is substantial at scale. Mitigated by
  per-tenant FinOps tagging (ADR-0174); tenants
  that want shorter retention can opt down.
- Build cost: this is ~6 person-months of substrate
  engineering across crypto, DNS automation, MTA
  configuration, dashboard, audit-chain ingestion.
  Not optional — it's the cost of being in the
  mail business.

### Risk register

- **R-1**: DKIM private key exfiltration. Mitigated
  by Cedar gating, audit-chain read logging, no
  human read path, emergency rotation in D-2.
- **R-2**: SPF flattening freshness lag. Mitigated
  by daily re-flatten + ESP API watch +
  72h fallback override.
- **R-3**: DMARC policy progression too aggressive.
  Mitigated by the conservative 14-day minimums and
  the council-deliverability override.
- **R-4**: BIMI logo or VMC compromise. Mitigated by
  CDN-bound serving (so we can rotate the SVG in
  minutes) and VMC revocation procedures bound to
  the CA's own process.
- **R-5**: ARC trusted-forwarder list compromise.
  Mitigated by quarterly review + audit-chain
  signing of the list.
- **R-6**: Complaint-rate automation false positive
  (an entire ISP marks a tenant as spam due to a
  shared-network event). Mitigated by per-MX
  bucketing in D-8 so an Apple-only spike doesn't
  lock the tenant out of Gmail.
- **R-7**: Inbound DMARC reject for a legitimate
  forwarder we missed. Mitigated by the ARC chain
  walk (D-7) and a tenant-side allow-list.

---

## Implementation surface

### Crates

- `oya-mail-dkim-sign` — outbound DKIM signer
  (Ed25519 + RSA-2048; consumes private keys from
  `cloud-secrets`; emits signing events to
  `audit-chain`).
- `oya-mail-dkim-verify` — inbound DKIM verifier.
- `oya-mail-spf-evaluate` — RFC 7208 evaluator with
  recursive cap and lookup counter.
- `oya-mail-spf-flatten` — recursive-include
  resolver + coalescer; emits the canonical
  flattened record.
- `oya-mail-dmarc-evaluate` — RFC 7489 evaluator,
  alignment logic, policy application.
- `oya-mail-dmarc-publish` — DMARC record emitter
  driven by the progression state machine.
- `oya-mail-dmarc-rua-ingest` — XML schema
  validator + canonical event projector.
- `oya-mail-bimi-publish` — BIMI selector
  publisher; SVG and VMC validator.
- `oya-mail-arc-sign` — outbound ARC sealer.
- `oya-mail-arc-verify` — inbound ARC chain walker.
- `oya-mail-deliverability-events` — schema-registry-bound
  `sent` / `delivered` / `bounced` / `complained` /
  `blocklist.hit` / `dmarc.aggregate` event shapes
  per ADR-0166.
- `oya-mail-deliverability-dashboard-projector` —
  event-stream projector feeding the read-side
  dashboard.
- `oya-mail-blocklist-probe` — concurrent
  DNSBL/URIBL/Postmaster-Tools probe.
- `oya-mail-warmup-controller` — MTA-queue
  rate-limiter driven by the warm-up ladder.
- `oya-mail-complaint-fbl-ingest` — feedback-loop
  parser for each subscribed FBL.
- `oya-mail-policy-cedar` — Cedar policy fragments
  binding mail-feature classes (per ADR-0140).

### Helm charts and infra

- `microservices/mail/iac/helm/mail-mta-outbound/` —
  Postfix-based outbound MTA pool. (We pick Postfix
  over Haraka or OpenSMTPD for the deliverability
  ecosystem familiarity; the same DKIM signer plugs
  in regardless.)
- `microservices/mail/iac/helm/mail-mta-inbound/` —
  inbound MTA pool with Rspamd sidecar.
- `microservices/mail/iac/helm/mail-reports/` —
  SMTP receiver for DMARC RUA / RUF and unsubscribe
  mailto: handlers.
- `microservices/cloud-network-dns/iac/helm/dns-orchestrator/`
  — extended to handle DKIM TXT publication and
  propagation probing.
- `microservices/cloud-secrets/iac/helm/openbao/` —
  extended with the DKIM-key path schema.

### Specs

- `/specs/microservices/mail.json` — extended with
  the deliverability section.
- `/specs/dns-automation.json` — DKIM and SPF
  publication contracts.
- `/specs/secrets-rotation.json` — DKIM rotation
  state machine.
- `/specs/event-schema-registry.json` — the
  deliverability events.
- `/specs/dmarc-rua-schema.json` — canonical event
  projection of the IETF RUA XML schema.

### Runbooks

- `microservices/mail/runbooks/dkim-rotation.md`
- `microservices/mail/runbooks/spf-flatten.md`
- `microservices/mail/runbooks/dmarc-progress.md`
- `microservices/mail/runbooks/bimi-publish.md`
- `microservices/mail/runbooks/blocklist-remediation.md`
- `microservices/mail/runbooks/warmup-restart.md`
- `microservices/mail/runbooks/complaint-spike-response.md`

### CI lanes

- `lean-a8-deliverability` (new lane) — verifies
  every PR touching mail crates:
  - Signs and verifies test messages against the
    Ed25519 and RSA-2048 test selectors.
  - Round-trips an SPF record through the
    flattener.
  - Round-trips a DMARC record through the
    publisher state machine.
  - Validates the BIMI SVG profile.
  - Parses and projects a fixture RUA report.
  - Round-trips an ARC chain through sign + verify.
- `lean-a5-doc-coverage` — verifies every decision
  in this ADR has a matching runbook and spec entry.
- `lean-a10-no-silent-regression` — protects the
  public DKIM/SPF/DMARC schema and the event-bus
  event shapes from breaking changes; any change
  requires an ADR amendment.

### Substrate dependencies

- `cloud-secrets` (ADR-0238) — DKIM key custody.
- `cloud-network-dns` (ADR-0240) — DNS publication.
- `audit-chain` (ADR-0028) — key generation,
  rotation, signing, and report events.
- `events-bus` + schema-registry (ADR-0123, ADR-0166)
  — deliverability event flow.
- `cedar policy engine` (ADR-0140, ADR-0243) —
  signing principal authorization.
- `intelligence substrate` (ADR-0255) — Rspamd
  Bayesian model hosting.
- `mail-reports` cell — DMARC RUA + RUF + unsubscribe
  mailto: ingestion.

---

## Verification

### Pre-promotion (Proposed → Accepted) gates

| Gate | Evidence required |
|---|---|
| G-1 | `oyatie-corp` outbound mail signs with both Ed25519 and RSA selectors and passes `dkim=pass` at Gmail, Outlook.com, Yahoo, Apple iCloud, Fastmail. Evidence: `evidence/adr-0273/dkim-multi-receiver.json` produced by the per-receiver probe. |
| G-2 | A synthetic stress tenant with 47 nested SPF includes flattens to <450 octets with <10 mechanisms. Evidence: `evidence/adr-0273/spf-flatten-stress.txt`. |
| G-3 | DMARC RUA ingestion live for 72h on `oyatie-corp` with at least 100 reports. Evidence: `evidence/adr-0273/rua-72h.json`. |
| G-4 | BIMI selector resolves and renders in Gmail and Apple Mail with a VMC-bound logo. Evidence: `evidence/adr-0273/bimi-render.png` (Gmail) + `bimi-render-apple.png`. |
| G-5 | ARC seal verified across a Google Groups → oyatie hop and an oyatie → external forward. Evidence: `evidence/adr-0273/arc-roundtrip.json`. |
| G-6 | Gmail complaint rate <0.1% over two consecutive weeks on `oyatie-corp` outbound. Evidence: Postmaster Tools screenshot bundle + signed RUA aggregate. |
| G-7 | DKIM key rotation runbook executed end-to-end on a non-production tenant; rotation overlap window observed; old selector RETIRED on schedule. Evidence: `evidence/adr-0273/dkim-rotation-trace.json`. |
| G-8 | Warm-up cadence enforced on a new domain — verify that a "blast on day 3" attempt was queued and rate-limited. Evidence: `evidence/adr-0273/warmup-trace.json`. |
| G-9 | Blocklist probe round-trips on all monitored lists. Evidence: `evidence/adr-0273/blocklist-probe.json`. |
| G-10 | Inbound auth pipeline correctly classifies all five outcomes (pass / softfail / fail / arc-pass / temperror) on a fixture set. Evidence: `evidence/adr-0273/inbound-auth-matrix.json`. |
| G-11 | Cedar-gated DKIM private-key read denial: a request as a non-`SigningPool` principal is rejected and audit-logged. Evidence: `evidence/adr-0273/cedar-deny-trace.json`. |
| G-12 | Multispectrum review v2.4.0 over the full ADR with all A1-A7 own-policy-adherence facets green. |

### Continuous verification (post-promotion)

- `lean-a8-deliverability` lane on every PR.
- Daily SPF flatten consistency probe on every tenant.
- Daily DMARC RUA ingestion sanity (no schema regressions).
- Hourly complaint-rate SLO probe.
- 10-minute blocklist probe.
- Monthly cross-receiver authentication audit (a
  test message from every active tenant sent to a
  fixed monitoring inbox at each major receiver;
  results scored).

### Failure modes covered by gates

- DKIM signature missing (G-1).
- SPF record too long (G-2).
- DMARC RUA ingestion silently broken (G-3).
- BIMI logo bound but not rendering (G-4).
- ARC chain broken across our own hops (G-5).
- Reputation collapse (G-6).
- Rotation gap (G-7).
- Cold-start blast (G-8).
- Blocklist hit unnoticed (G-9).
- Inbound auth misclassification (G-10).
- Private-key access escalation (G-11).
- Own-policy drift (G-12).

---

## References

### IETF RFCs

- RFC 6376 — DomainKeys Identified Mail (DKIM) Signatures.
  https://www.rfc-editor.org/rfc/rfc6376
- RFC 6377 — DKIM and Mailing Lists. (Informational; informs ARC design.)
  https://www.rfc-editor.org/rfc/rfc6377
- RFC 7208 — Sender Policy Framework (SPF) for Authorizing Use of Domains in Email, Version 1.
  https://www.rfc-editor.org/rfc/rfc7208
- RFC 7489 — Domain-based Message Authentication, Reporting, and Conformance (DMARC).
  https://www.rfc-editor.org/rfc/rfc7489
- RFC 7960 — Interoperability Issues between DMARC and Indirect Email Flows. (Informational; ARC justification.)
  https://www.rfc-editor.org/rfc/rfc7960
- RFC 8058 — Signaling One-Click Functionality for List Email Headers.
  https://www.rfc-editor.org/rfc/rfc8058
- RFC 8460 — SMTP TLS Reporting. (Out of scope for this ADR; a future inbound TLS posture ADR will pick it up.)
  https://www.rfc-editor.org/rfc/rfc8460
- RFC 8461 — SMTP MTA Strict Transport Security (MTA-STS). (Out of scope this ADR.)
  https://www.rfc-editor.org/rfc/rfc8461
- RFC 8463 — A New Cryptographic Signature Method for DKIM (Ed25519).
  https://www.rfc-editor.org/rfc/rfc8463
- RFC 8616 — Email Authentication for Internationalized Mail.
  https://www.rfc-editor.org/rfc/rfc8616
- RFC 8617 — The Authenticated Received Chain (ARC) Protocol.
  https://www.rfc-editor.org/rfc/rfc8617
- RFC 9695 — Brand Indicators for Message Identification (BIMI). (2024.)
  https://www.rfc-editor.org/rfc/rfc9695
- RFC 6409 — Message Submission for Mail.
  https://www.rfc-editor.org/rfc/rfc6409
- RFC 5321 — Simple Mail Transfer Protocol.
  https://www.rfc-editor.org/rfc/rfc5321
- RFC 5322 — Internet Message Format.
  https://www.rfc-editor.org/rfc/rfc5322

### Receiver published policies

- Gmail Email Sender Guidelines (Bulk Sender, 2024 revision).
  https://support.google.com/mail/answer/81126
- Yahoo Sender Hub Bulk Sender Requirements (2024).
  https://senders.yahooinc.com/best-practices/
- Microsoft 365 Outbound Sender Guidance and Bulk Compliance (2025 update).
  https://learn.microsoft.com/en-us/microsoft-365/security/office-365-security/outbound-spam-policies-configure
- Apple Hide-My-Email Developer Documentation (2024).
  https://developer.apple.com/icloud/hide-my-email/
- Apple Mail Privacy Protection (Mail Privacy Protection, MPP). Apple Mail; iOS 15+ / macOS 12+.
- Google Postmaster Tools.
  https://postmaster.google.com/
- Microsoft SNDS + JMRP.
  https://sendersupport.olc.protection.outlook.com/snds/
  https://sendersupport.olc.protection.outlook.com/pm/

### Standards bodies and blocklists

- M3AAWG (Messaging, Malware, Mobile Anti-Abuse Working Group) ARC and bulk-sender guidance.
  https://www.m3aawg.org/published-documents
- Spamhaus Project. https://www.spamhaus.org/
- SURBL. https://www.surbl.org/
- URIBL. https://uribl.com/
- AuthIndicators Working Group (BIMI). https://bimigroup.org/

### NIST and crypto guidance

- NIST SP 800-57 Part 1 Rev 5 — Recommendation for Key Management: Cryptoperiods.
- NIST SP 800-131A Rev 2 — Transitioning the Use of Cryptographic Algorithms and Key Lengths.

### Internal references

- microservices/mail/PRD.md
- ADR-0201 — Email + transactional comms adapter substrate.
- ADR-0208 — Mail microservice substrate.
- ADR-0210 — Mail JMAP / IMAP / SMTP protocol surface.
- ADR-0238 — OpenBao secrets storage.
- ADR-0240 — Sovereign cloud per-regional pack.
- ADR-0242 — Oyatie-is-a-tenant doctrine.
- ADR-0243 — Cedar as universal gate.
- ADR-0245 — Substrate vs product layering.
- ADR-0251 — Compliance pack cell certification levels.
- ADR-0255 — Intelligence as two-layer AI substrate.

---

## Appendix A — Pattern attribution

The decisions here borrow patterns from several
well-established practitioners. We acknowledge the
sources so future engineers can read the originals
when refining behavior, and so we don't
silently reinvent thinking that has a paper trail.

### A.1 — DKIM dual-algorithm rollout pattern

Borrowed from Fastmail's published 2020 blog post on
their Ed25519 rollout strategy, and from
Cloudflare's 2021 deliverability writeup on
maintaining RSA fallback during the Ed25519
ecosystem catch-up phase. Both vendors documented
that publishing both selectors is the only safe
approach during a multi-year ecosystem transition;
we adopt that as canonical.

### A.2 — 90-day rotation cadence

Borrowed from the LetsEncrypt 90-day certificate
rotation model, with the same operational argument:
short enough that key compromise has a bounded
window, long enough that automation overhead is
not a daily concern. Mailchimp and SparkPost both
adopted similar quarterly DKIM rotation by the
mid-2020s; we follow that consensus.

### A.3 — SPF flattening

Borrowed from the long industry practice
codified by Valimail, Dmarcian, and EasyDMARC.
The "flatten plus fallback" pattern is theirs;
we extend it with daily re-flatten and an
explicit fallback record so the override path
is operator-friendly.

### A.4 — DMARC progression ladder

Borrowed from M3AAWG's DMARC Implementation
Guide and from Google's Postmaster Tools
guidance. The 14-day minimums at each rung are
M3AAWG's; the per-tenant override mechanism
(council-deliverability four-eyes) is our own.

### A.5 — BIMI selector model

Borrowed from the AuthIndicators Working Group
RFC 9695. The "substrate-owned CDN serving the
SVG and VMC" pattern is our own optimization,
adapted from how Cloudflare host BIMI logos for
their own customers.

### A.6 — ARC trusted-forwarder list

Borrowed from Google's published ARC trust
list and the M3AAWG ARC trust signal exchange
working draft. Quarterly review cadence is
our own choice, calibrated to the empirical
rate of new mailing-list / group-mail
introductions in the ecosystem (slow).

### A.7 — Complaint-rate automation

The 0.3% Gmail threshold and the 0.1% target
both come directly from Gmail's published
bulk-sender requirements. The graduated
response — pause Gmail, then pause all
bulk receivers, then move to penalty IP pool —
is our own design, inspired by how SendGrid's
"reputation manager" responds to similar
events.

### A.8 — Per-tenant warm-up cadence

Borrowed from the well-known warm-up curve
documented by Return Path, Validity, and 250ok.
The specific day-counts (50/200/1000/5000) are
calibrated to our target volume profile and may
be tuned per receiver class as we accumulate
reputation data.

### A.9 — Blocklist remediation workflow

Borrowed from M3AAWG's "Operational Procedures
for Anti-Abuse Reporting and Disposition" with
adaptations for our substrate's audit-chain
requirements.

### A.10 — Apple Hide-My-Email handling

Borrowed from Apple's own developer documentation
and the WWDC 2021 session "Meet Mail Privacy
Protection". The "ephemeral alias, not CRM key"
pattern is our own response to the Apple
documentation's explicit guidance that
Hide-My-Email aliases are not stable identifiers.

### A.11 — Apple Mail privacy protection handling

Borrowed from Litmus and Mailchimp's published
analyses of Apple Mail Privacy Protection
behavior. The "exclude Apple Mail user agents
from open-rate computation entirely" rule is
the only honest response.

---

## Appendix B — Worked example: bringing the `acme` tenant online

This appendix walks through the lifecycle of a real
tenant adopting oyatie Work Mail, showing how each
decision fires.

### B.0 — Pre-conditions

- Acme is migrating from Microsoft 365.
- Their primary mail domain is `acme.com`.
- They have marketing subdomains `marketing.acme.com`
  (currently sent via SendGrid) and
  `receipts.acme.com` (currently sent via Postmark).
- They want to keep both ESPs running in parallel for
  90 days during cutover.
- They have a registered trademark and intend to
  pursue a BIMI VMC.

### B.1 — Day 0: tenant provisioning

The tenant onboarding workflow runs. The mail
provisioning step:

1. Generates Ed25519 + RSA-2048 DKIM key pairs for
   `acme.com`. Selectors `oya-ed-2026q2-001` and
   `oya-rsa-2026q2-001`. Stored in
   `secret/data/tenants/acme/mail/dkim/acme.com/{ed25519,rsa-2048}/`.
2. Generates the same for `marketing.acme.com` and
   `receipts.acme.com`.
3. Computes the canonical SPF record for `acme.com`
   including (a) the oyatie outbound IP pool, (b)
   SendGrid (`include:sendgrid.net`), (c) Postmark
   (`include:spf.mtasv.net`), (d) flattens.
4. Computes the SPF for `marketing.acme.com`
   (SendGrid only) and `receipts.acme.com`
   (Postmark only).
5. Publishes:
   - DKIM TXT records (both selectors, both
     algorithms, all three domains) via
     `cloud-network-dns`.
   - SPF flattened record at `acme.com` and
     unflattened fallback at `_spf-fallback.acme.com`.
   - DMARC record at `_dmarc.acme.com` with
     `p=none; rua=mailto:dmarc-rua@reports.acme.oyamail.io;
     ruf=mailto:dmarc-ruf@reports.acme.oyamail.io;
     fo=1; adkim=r; aspf=r; pct=100`.
   - DMARC subdomain policy same as parent (default).
6. Probes propagation; flips keys ACTIVE only after
   all 8 public resolvers return correct payloads.

Total wall-clock: 15-25 minutes depending on the
authoritative DNS provider's propagation behavior.

### B.2 — Days 1-7: observation + warm-up

Outbound from oyatie's MTA pool is rate-limited per
D-10:
- Day 1: 50 messages to Acme's own employees.
- Days 2-3: 200 messages/day to employees + opt-in
  cohort.
- Day 4-7: 1000 messages/day, ramping toward 25% of
  Acme's real list.

Meanwhile, SendGrid and Postmark continue sending
their portion of Acme's mail. The DMARC RUA
ingestion (D-5) starts seeing reports from
receivers: most show `dmarc=pass` because SendGrid
and Postmark sign their portion with their own
keys aligned at the subdomain level.

The dashboard surfaces:
- Acme's current `dmarc=pass` rate per source.
- Any source IP that fails alignment (e.g., an
  internal mail server Acme forgot to retire).

### B.3 — Days 14-28: progress to `p=quarantine`

After 14 days of `p=none` with >99.5% alignment
on the observed traffic, the DMARC progression
workflow flips `p=quarantine`. Dashboard banner
announces the change to the Acme admin team. They
have not had to do anything; the workflow ran
itself.

BIMI eligibility unlocks at this point (per D-6).
The Acme admin uploads their SVG logo and VMC. The
publisher validates and publishes
`default._bimi.acme.com`.

### B.4 — Day 42-56: progress to `p=reject`

After another 14 days of `p=quarantine` with
>99.9% alignment, the workflow flips `p=reject`.
SPF mechanism on `acme.com` flips from `~all`
(softfail) to `-all` (hardfail).

The `_spf-fallback.acme.com` record stays
published as a rollback safety net.

### B.5 — Day 90: first DKIM rotation

90 days after provisioning, the rotation workflow
runs (D-2):

- T-7: generates `oya-ed-2026q3-001` and
  `oya-rsa-2026q3-001` selectors. Publishes to
  DNS. Verifies propagation.
- T+0: flips them ACTIVE; old selectors DEPRECATED.
  All new outbound mail signs with the new keys;
  old DNS records stay published.
- T+7: old selectors RETIRED. DNS records removed.
  Private keys purged from `cloud-secrets`.

The Acme admin sees a single dashboard entry:
"DKIM rotation 2026q2 → 2026q3 completed
successfully". They didn't have to do anything.

### B.6 — Day 120: complaint spike

A marketing campaign goes out and produces a 0.45%
complaint rate on Gmail recipients in the first
hour. The automation (D-9):

1. Pauses Acme's outbound to Gmail-class recipients
   immediately.
2. Dashboard banner: "Gmail complaint rate
   exceeded — outbound paused, please investigate
   in dashboard".
3. The deliverability dashboard surfaces the
   offending campaign — content, send time, list
   segment, complaint rate per segment.

The Acme admin reviews, identifies a list
segmentation bug (campaign went to opted-out
recipients), fixes it, acknowledges the alert. The
warmup-restart workflow runs Acme back through the
day-15-equivalent ramp on Gmail recipients before
re-opening the full pipe.

### B.7 — Day 150: blocklist hit (false positive)

Spamhaus SBL lists Acme's outbound IP for 90
minutes due to a misclassified content pattern.
The blocklist probe (D-11) detects within 10
minutes:

1. On-call deliverability operator paged.
2. Evidence bundle assembled.
3. Spamhaus delisting request submitted via API.
4. Acme's outbound to bulk recipients throttled to
   10 msg/sec from the affected IP.
5. Spamhaus delists at minute 90; throttle
   continues per the cooldown ladder until
   complaint rates confirm safety.

### B.8 — Day 365: annual security review

The annual security review covers, per Acme's
tenant compliance overlay:

- DKIM key rotation history (4 successful
  rotations).
- DMARC RUA report archive (18 months by default,
  retained 365 days under Acme's KR-PIPA
  retention floor since Acme is a Korean entity).
- Complaint-rate trendline (mostly below 0.05%,
  one incident at day 120 fully remediated).
- Blocklist history (one false positive at day
  150, fully remediated).
- BIMI VMC validity (Acme's VMC is good for
  another two years).

The review produces no findings; the substrate
ran the deliverability stack with no human
intervention beyond the day-120 acknowledgment.

### B.9 — Acme leaves oyatie (hypothetical)

When Acme's tenant is deprovisioned:

1. DMARC record updated to `p=reject` with
   `rua=mailto:dmarc-rua@reports.acme.com` (their
   own endpoint) for 30 days, giving them a
   transition window.
2. DKIM selectors marked DEPRECATED so the
   keys cannot sign new mail but in-flight
   verification still works.
3. After 30 days, DKIM keys RETIRED; SPF record
   removed; BIMI record removed; DMARC reverts to
   whatever Acme's new provider publishes.
4. Audit-chain retains the public key
   fingerprints and the full rotation history
   forever (per ADR-0028 immutability).
5. Acme's private keys are zeroized in
   `cloud-secrets`.

This appendix is a normative reference: any
deviation observed in production constitutes a
deliverability incident under D-8 and triggers an
audit-chain investigation.

---

## Glossary

- **ARC** — Authenticated Received Chain (RFC 8617).
- **BIMI** — Brand Indicators for Message Identification (RFC 9695).
- **CMC** — Common Mark Certificate (BIMI variant for prior-use marks).
- **DKIM** — DomainKeys Identified Mail (RFC 6376, RFC 8463).
- **DMARC** — Domain-based Message Authentication, Reporting, and Conformance (RFC 7489).
- **FBL** — Feedback Loop (recipient-side complaint feed).
- **JMRP** — Junk Mail Reporting Partner program (Microsoft).
- **MPP** — Mail Privacy Protection (Apple Mail).
- **MTA-STS** — SMTP MTA Strict Transport Security (RFC 8461).
- **RUA** — Reporting URI for Aggregate reports (DMARC tag).
- **RUF** — Reporting URI for Forensic reports (DMARC tag).
- **SBL/XBL/PBL/DBL** — Spamhaus blocklist subsets.
- **SNDS** — Smart Network Data Services (Microsoft).
- **SPF** — Sender Policy Framework (RFC 7208).
- **TLS-RPT** — SMTP TLS Reporting (RFC 8460).
- **VMC** — Verified Mark Certificate (BIMI).
