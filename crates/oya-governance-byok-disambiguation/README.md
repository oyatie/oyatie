# oya-governance-byok-disambiguation

Enforces ADR-0255 §D-4 and ADR-0251 §D-10 terminology separation.

## Rule

Markdown under `docs/` and `microservices/` may not use bare `BYOK` when the meaning is unclear. Each reference must classify as `provider-BYOK` for external provider credentials, `encryption-BYOK` for cryptographic keys, or explicitly contrast both.

## Trigger

Run when documentation or implementation packets mention BYOK.

```bash
cargo run --manifest-path crates/oya-governance-byok-disambiguation/Cargo.toml -- --root . --strict
```

## Compliant Output

```text
ADR-0255-D4+ADR-0251-D10: Passed (1 markdown files, 1 BYOK references, 0 violations)
OK: every BYOK reference is disambiguated.
```

## Violation Output

```text
docs/security.md:1:31: AmbiguousByok: BYOK reference does not say provider-BYOK or encryption-BYOK
  excerpt: Enterprise tenants can enable BYOK during onboarding.
  fix: Use provider-BYOK (ADR-0255 §D-4) for external provider credentials/API keys. Use encryption-BYOK (ADR-0251 §D-10) for KMS, KEK, CMK, HSM, or envelope-encryption keys.
```

## How To Fix

Use `provider-BYOK (ADR-0255 §D-4)` when discussing external provider credentials and `encryption-BYOK (ADR-0251 §D-10)` when discussing KMS/HSM/KEK/CMK ownership.
