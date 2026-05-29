---
doc_id: finops-portal/threat-model
authored: 2026-05-18
status: ready
authority: ADR-0183 cedar-policy-discipline + ADR-0162 audit-log integrity
classification: internal
---

# Threat model — finops-portal

This is a STRIDE-based threat model for `finops-portal`. Scope: the
µservice's HTTP + gRPC surfaces, its data plane, and the credit-
ledger + cost-allocation-policy editor surfaces. Out of scope: the
upstream OpenCost / Mimir / SeaweedFS systems (they have their own
models).

## Asset inventory

1. **Tenant invoice data** — per-tenant cost decomposition, credit
   ledger entries, dollar amounts. High business confidentiality.
2. **Cost-allocation policies** — fleet-wide / pack-wide / tenant
   rules that govern who-pays-for-what. Tampering shifts cost.
3. **Quarterly regulator evidence** — signed envelopes. Tampering
   creates a regulatory finding.
4. **Audit-chain seal events** — `TenantInvoiceFinalized`,
   `CostAllocationPolicyChanged`, `CreditApplied`, etc. Tamper
   means audit-log forgery.
5. **Cedar policy bundle** — runtime authz rules. Tamper means
   privilege escalation.
6. **Signed-URL HMAC key** — issues short-lived Grafana embed +
   FOCUS download URLs. Tamper means URL forgery.
7. **Ed25519 quarterly signing key** — signs regulator envelopes.
   Tamper means signature forgery.

## Trust boundaries

| Boundary             | Inside (trusted)              | Outside (untrusted)             |
|----------------------|-------------------------------|---------------------------------|
| Process              | finops-portal-app process     | network                         |
| Cluster              | finops-portal pods + sidecars | tenant browsers, regulator browsers |
| Tenant scope         | one tenant's data             | other tenants' data             |
| Pack scope           | one regulatory pack           | other packs (no cross-read)     |
| HSM                  | quarterly signing key         | application code                |

## STRIDE per asset

### 1. Tenant invoice data

- **Spoofing**: principal claims to be another tenant.
  Mitigation: JWT verification + Cedar `tenant_id` equality check
  (`policy/cedar/tenant-isolation.cedar`).
- **Tampering**: a finalized invoice is modified post-seal.
  Mitigation: invoice is append-only after `Finalize`; audit-chain
  seal envelope hash detects mutation; quarterly emit reconciler
  detects drift.
- **Repudiation**: a tenant denies they saw an invoice.
  Mitigation: every render emits a `TenantInvoiceRendered`
  audit-chain event with the principal's hashed identity.
- **Information disclosure**: invoice data leaks across tenants.
  Mitigation: Cedar deny-by-default; PHI redaction for US-
  healthcare; OpenAPI examples carry synthetic data only.
- **Denial of service**: invoice render endpoint flooded.
  Mitigation: per-tenant rate limit (60/min, 1000/h per capability
  declaration); HPA elastically scales 3..12 replicas.
- **Elevation of privilege**: tenant admin gains ops-finops scope.
  Mitigation: `principal.tenant_scope` is a JWT claim signed by
  the auth service; finops-portal does not re-issue it.

### 2. Cost-allocation policies

- **Tampering**: a malicious actor promotes a fleet-scope policy
  that misallocates cost.
  Mitigation: 2-reviewer quorum enforced at the domain layer
  (IP-010); audit-chain seal of `CostAllocationPolicyChanged`;
  24h alert window for fleet-impact (`runbooks/cost-allocation-
  policy-rollback.md`).
- **Repudiation**: reviewer denies they approved a policy.
  Mitigation: `Reviewer.approved_at` is recorded with the
  reviewer's principal id; sealed.

### 3. Quarterly regulator evidence

- **Tampering**: an envelope is modified after sealing.
  Mitigation: Ed25519 signature; verifier checks against the
  published key.
- **Information disclosure**: cross-pack regulator reads
  another pack's data.
  Mitigation: Cedar `regulator-evidence-emit.cedar` enforces
  pack + residency match.

### 4. Cedar policy bundle

- **Tampering**: a malicious deploy slips a permissive Cedar
  policy.
  Mitigation: policy files live in repo and ship in container;
  CI lane `lean-a8-cedar-policy-validate` validates schema +
  unit tests; PR review required.

### 5. Signed-URL HMAC key

- **Spoofing / tampering**: a leaked key issues fake URLs.
  Mitigation: key in secret store; rotated quarterly; envelope
  carries key fingerprint so verifier detects mismatch.

### 6. Ed25519 quarterly key

- **Tampering**: an attacker signs a fake envelope.
  Mitigation: private key in HSM; only emit job can sign;
  rotation quarterly; old keys remain verifiable via the
  audit-chain-published `FinOpsQuarterlyKeyPublished` event.

## Top-5 risks (residual)

1. **Cedar policy drift between file and runtime**: addressed by
   unit test in IP-007.
2. **Tenant-id leak via Debug impl**: addressed by redacted
   `Debug` on `TenantInvoice` (IP-001 acceptance §criterion 5).
3. **Idempotency hole on `finalize_invoice` + audit-chain race**:
   addressed by quarterly reconciler.
4. **Signed-URL replay**: 5min TTL caps blast radius; logged.
5. **Cross-pack regulator read**: addressed by double-guard
   forbid clauses in Cedar.

## Verification

- STRIDE matrix per asset (above) is reviewed quarterly.
- Cedar policy unit tests cover all 6 cross-boundary scenarios.
- Threat model is sealed to audit-chain on quarterly review.

## References

- ADR-0183 cedar-policy-discipline.
- ADR-0162 per-tenant audit-log slicing.
- `policy/cedar/*.cedar`.
- `dpia.md`.
- `compliance-matrix.md`.
