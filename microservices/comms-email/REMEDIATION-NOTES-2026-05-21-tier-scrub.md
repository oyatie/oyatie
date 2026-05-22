# Comms Email Tier Scrub Remediation Notes

Date: 2026-05-21

## Scope

Wave 15J-batch-4 BUCKET-03 scrub for `microservices/comms-email`.

## Files Modified

- `README.md` — 34 lines
- `manifest.json` — 148 lines
- `coherence-audit-2026-05-20.md` — 639 lines
- `feature-parity-matrix-2026-05-20.md` — 430 lines
- `performance-benchmark-numbers-2026-05-20.md` — 405 lines
- `benchmarks/comms-email-vs-sendgrid-vs-ses-vs-postmark.md` — 111 lines
- `tutorials/send-1m-transactional-campaign-with-warmup.md` — 268 lines
- `faqs/deliverability-engineer-faq.md` — 106 lines

## Deletion

`capability-tiers/` deleted: Y.

## Replacement Count

Rough replacement count: ~25 content replacements plus the directory deletion.

## Design Decisions

- Replaced tiered deliverability, DKIM, warmup, and send-envelope language with
  `tenant_class`, compliance-pack, cell-topology, and paid billing-component
  phrasing.
- Preserved compliance-pack-specific HSM and sovereign-pack substance while
  removing Bronze/Silver/Gold/Platinum labels.
- Replaced benchmark labels with paid tenant-class or deployment-context
  labels instead of commercial capability tiers.
- Updated the README to cite ADR-0330 and the canonical `tenant_class` plus
  `billing_components` model.

## Outstanding Follow-Ups

None for the assigned Bronze/Silver/Gold/Platinum and `capability_tier` scrub.

## Wave 15-IP-substance scrub (2026-05-21)

Scope: `microservices/comms-email` IP files only.

Inventory:

- 36 implementation-plan files inspected: IP-001 through IP-026 plus journey IPs j91-j100.
- Preserved as substantive: IP-001 through IP-015.
- Stamped/thin shells detected and rewritten in place: IP-016 through IP-026.
- Journey row-flood shells detected during bucket verification and rewritten in place:
  IP-journey-j91 through IP-journey-j100.
- Deleted: none.

Rewrite basis:

- Service contracts cited only from existing files:
  `contracts/openapi.yaml`, `contracts/asyncapi.yaml`, and `contracts/comms_email.proto`.
- Capability anchors cited only from existing files:
  `capabilities/T1-bounce-handle.json`, `capabilities/T2-list-manage.json`,
  `capabilities/T3-inbound-receive.json`, and related T0/T1/T2 files.
- Policy anchors cited only from existing files under `policy/`, especially
  `abuse-defence.cedar`, `action-authorization.cedar`,
  `comms-email-suppression-list.cedar`, and `data-residency.cedar`.
- Operational counterparts cited only from existing runbooks and dashboards under this
  service.

Notes:

- IP-023, IP-024, and IP-025 now explicitly state that their REST surfaces are contract
  deltas against `contracts/openapi.yaml`; they do not claim those routes already exist.
- IP-026 now uses the real `contracts/asyncapi.yaml` suppression channel and reason enum
  instead of invented unsubscribe channel claims.
- IP-journey-j91 through IP-journey-j100 now bind to real comms-email contracts,
  policy files, dashboards, SLOs, and runbooks instead of generated `TASK-###`,
  `FM-###`, and generic invariant rows.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:

None. Assigned inventory found zero Redis references in `microservices/comms-email`.

Counterpart-fact preservations:

None.

Files renamed (git mv):

None.
