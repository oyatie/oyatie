# SDK plan — `comms-email` µservice

> ADR anchors: ADR-0201, ADR-0145, ADR-0166.

## 1. SDK target audience

Every oyatie µservice that needs to send transactional email.
The SDK is the trait `EmailComms` from
`crates/oya-shared-email-comms-kernel` plus a thin idiomatic
wrapper.

## 2. Surface area

```
let comms: Box<dyn EmailComms> = comms_factory(active_provider);
let outcome = comms.send(&binding, &message)?;
```

Helpers:

- `comms_factory(provider) -> Box<dyn EmailComms>` —
  config-driven adapter selection.
- `bind_tenant(tenant_id, from_domain) -> DeliverabilityBinding` —
  resolves the DKIM binding from OpenBao + provider state.
- `compose(template_id, locale, vars) -> OutboundMessage` —
  MJML compile + Liquid sub.

## 3. Language coverage

- Rust (canonical, this batch).
- Other client stacks (Apple Swift, Kotlin Android, etc.) call
  the µservice over REST per IP-001 contracts.

## 4. Versioning

- SemVer-locked at the workspace level.
- Adapter additions are minor; trait surface changes are major.
- ADR-0145 (event schema) versioning controls webhook contract.

## 5. Test fixtures

- A `comms-email-test` fixture crate provides:
  - `MockEmailComms` implementing the trait — for unit tests
    that need to assert send behavior without provider calls.
  - DKIM key fixtures + DNS fixtures for offline tests.

> Note: `MockEmailComms` is a *test fixture*, not a Noop
> production adapter. Production callers never see it.

## 6. Documentation

- API reference under `microservices/comms-email/contracts/`.
- Tutorial: "Send your first email" in
  `docs/standards/` (Tier 1 mdbook) + Backstage TechDocs.
- Sample code in `examples/comms-email/`.

## 7. Backwards compatibility

- Adapter shells (no-Noop fallback) ship today; future
  provider additions land as additional adapter sub-crates.
- Trait additions go through ADR cadence with a major-bump
  if the addition breaks existing impls.

## 8. Migration

- Existing one-off SES integrations migrate to the SDK behind
  the `EmailComms` trait.
- Migration ADR addendum (T+30d) enumerates call sites and
  cutover order.

## 9. Open questions

- Async vs sync trait posture: ADR-0145 client kernel uses
  sync; the email kernel currently also sync. A future async
  variant may land if integration patterns require it.
