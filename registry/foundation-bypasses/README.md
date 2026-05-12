# Foundation gate exception ledger

This directory is intentionally present so the `foundation-bypass` gate can distinguish an explicit empty exception ledger from a missing control surface.

Add one `*.yaml` file per approved, expiring exception. Empty means no active or remediated exceptions are recorded.

Supported record classes:

- `foundation-bypass` (default): an expiring foundation gate exception with `byp_` id, PR reference, affected crate, gate name, actor, rationale, and remediation window.
- `autonomy-break-glass`: an ADR-0022 autonomy override record with `abg_` id, tenant/capability, requested/permitted tier, distinct M-of-N approvers, explicit expiry, and optional revocation day.

Validate with:

```bash
oya gate validate foundation-bypass --ledger registry/foundation-bypasses
```
