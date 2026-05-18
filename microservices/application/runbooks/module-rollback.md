---
doc_class: Runbook
title: Module Rollback — revert product bundle to prior signed version
microservice: application
severity: "Sev-1 (integrity failure) / Sev-2 (operational)"
status: Accepted
owner_team: axis-application + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/application/failure-modes.md (FM-02, FM-04, FM-10)
  - microservices/application/incident-response.md
  - microservices/application/threat-model.md (S-06, T-02, R-03)
doc_status: published
---

# Runbook: Module Rollback

## Trigger

ONE of:

1. **FM-02 module integrity failure** — `oya_application_module_signature_invalid_total > 0` (automated; Sev-1 page).
2. **FM-10 hydration regression** — Lighthouse synthetic fail + TTI breach (SLO worker auto-invokes if budget exceeded).
3. **Manual** — IC declares rollback after security event.

## Severity

- Integrity failure: **Sev-1**.
- Operational revert (no breach): **Sev-2**.

## Pre-checks

1. Confirm the failing module: `oya_application_module_signature_invalid_total{module="...", version="..."}`.
2. Confirm the prior good version: query `oya_application_module_version_prior{module="..."}`; verify its `signer_key_id` matches the registered publisher key in OpenBao.
3. Verify the prior version's eligibility verdict at staging was `eligible` at the time it was promoted.
4. Confirm prior version's content hash matches the manifest SRI hash present in shell HTML.
5. If signer-key compromise suspected: ops-security PrivacyLead joins immediately; rotate publisher key before revert.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>`; assign IC + SME (axis-application) + ops-security if integrity-failure | ≤ 5 min |
| 2 | Confirm pre-checks | ≤ 2 min |
| 3 | Revert module pointer: `cargo run -p oya-dev-cli -- application module revert --module <name> --to-version <prior-version> --reason "<rfc>"`. The CLI: <br>a. validates the prior version's Ed25519 signature against the publisher key;<br>b. updates `module_version_pointer` row in Postgres for the module's tenant scope;<br>c. emits `ModuleLoadRejected` event for the bad version + `ModuleLoaded` event for the prior;<br>d. triggers a CDN purge for the bad version's URL (see `cdn-purge.md`);<br>e. seals the change in audit-chain. | ≤ 60 s |
| 4 | Verify pointer flip: `oya_application_module_version_active{module="..."} == <prior-version>`. | ≤ 1 min |
| 5 | Verify CDN purged the bad version | ≤ 60 s (per `cdn-purge.md`) |
| 6 | Trigger synthetic probe: load the module in a canary tenant; verify it instantiates + hydrates | ≤ 5 min |
| 7 | If publisher-key compromise suspected: rotate via `cargo run -p oya-dev-cli -- application module rotate-publisher-key --module <name>` and force re-publish | ≤ 30 min |
| 8 | CommsLead: tenant + status page comms | ≤ 30 min |
| 9 | If Sev-1: PrivacyLead initiates regulatory-notification chain | per timeline |
| 10 | Postmortem within 5 BDs | – |

## Roll-forward (if rollback itself fails)

Rare: prior version itself has a known regression. Skip back two versions
via `--to-version <N-2>` and notify the affected product team's owner to
prepare a new fix bundle.

## Verification

- `oya_application_module_version_active{module="..."}` reflects the rollback.
- `oya_application_module_signature_invalid_total == 0` for ≥ 5 min.
- Synthetic probe instantiates module successfully.
- Audit-chain seal log contains `ModuleRolledBack` event with reason.
- Status page reflects "Resolved" with rollback timestamp.

## Publisher-key rotation procedure

If the rollback was triggered by suspected publisher-key compromise:

1. Operator with OpenBao admin grant: `cargo run -p oya-dev-cli -- application module rotate-publisher-key --module <name>`.
2. The CLI: (a) generates a new Ed25519 keypair via OpenBao transit engine; (b) registers the new public key as `expected_signer_key_id` in `Module` Ontology entity; (c) revokes the prior key (immediately invalidates any future signed manifest by the prior key).
3. Notify product team to re-publish their bundle signed with the new key.
4. Old key remains in OpenBao audit log for forensics.

## Post-incident updates

- Postmortem published.
- Action items: "why did publisher key reach compromise?", "is per-product key separation strict enough?".
- Update this runbook if procedure missed a step.

## References

- `failure-modes.md` FM-02, FM-04, FM-10.
- `threat-model.md` S-06 (publisher impersonation), T-02 (manifest tamper), R-03 (publisher repudiation).
- ADR-0028 audit chain.
