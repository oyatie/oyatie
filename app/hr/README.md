# HR

Owner: `app/hr`

Status: portable-app migration; domain foundation only

HR is the tenant-portable People and employment application. It owns employee
and employment records, organization/manager relations, onboarding readiness,
leave policy projections, labor-compliance decisions, sensitive-read policy,
and HR evidence references.

The landed Rust domain and in-process adapters are test foundations. They do
not yet constitute a durable service, sold network facade, installed-pack
integration, downstream delivery path, or measured SLO. The current direct
Data/Gateway dependencies and volatile storage are migration debt.

Canonical owner law:

- [ADR.md](ADR.md) — decisions and portability boundaries
- [PRD.md](PRD.md) — product requirements, acceptance, and SLO objectives
- [SPEC.md](SPEC.md) — current contract and target transaction/fault semantics
- [PLAN.md](PLAN.md) — L2a through L2k.3 implementation, retirement, and
  bounded-production sequence

Replay obtains its bounded, provider-authenticated generation matrix through the
record-encryption port, uses only the returned generation-scoped opaque authority
to derive a candidate, then authenticates/decrypts a located candidate and
constant-time compares canonical plaintext in memory. Ciphertext equality is
never replay equality; plaintext never persists outside authenticated envelopes.
The executable baseline is canonical-request V1 only: a second format cannot be
advertised, selected for writes, or required by replay/rekey until a separately
accepted format-lifecycle decision supplies its codec, authority, migration, and
independent-oracle closure. Keyring membership is provider-authoritative and
frozen into a rotation fence, so a missing repository cannot be omitted before
an old generation is revoked. Required-authority outages fail closed and consume
availability budget for eligible traffic until recovery or acknowledged routing
withdrawal.

HR does not own payroll calculation/disbursement, accounting, workflow
execution, audit-chain persistence, IAM/PDP, Data/Storage/Gateway engines,
notification delivery, or deployment infrastructure. Those effects cross
HR-owned ports and replaceable adapters.
