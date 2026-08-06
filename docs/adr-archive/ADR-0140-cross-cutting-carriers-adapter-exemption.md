---
id: ADR-0140
status: Superseded
deciders: council-architecture, council-product, council-privacy, axis-network, axis-meet, axis-mail, axis-messenger, axis-calendar, axis-drive, axis-recordings, ops-sre-reliability, ops-security
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0145]
related: [ADR-0011, ADR-0056, ADR-0064, ADR-0105, ADR-0123, ADR-0131, ADR-0132, ADR-0135, ADR-0136, ADR-0139]
related_memory:
  - feedback_workflow_objectgraph_adapter_layer
  - feedback_workflow_is_shared
  - feedback_canonical_base_localization
  - feedback_no_silent_regression
  - feedback_quality_performance_scalability_bar
  - feedback_flat_product_catalog
related_specs:
  - /specs/per-microservice-flat-layout.json
  - /specs/microservices/messenger.json
  - /specs/microservices/mail.json
  - /specs/microservices/calendar.json
session_context:
  authored: 2026-05-18
  integration_review_finding: INT-003 — systemic gap; multiple µservice
    networkpolicy.yaml templates carry direct egress to drive / mail /
    messenger / calendar / recordings on behalf of attachment / share /
    notify / bind / store flows, none of which are routed through the
    workflow-engine event-bus. Treating these flows as adapter-rule
    violations would force every file-share, calendar-invite, channel-mention
    and recording-store through a single chokepoint, defeating workflow-engine's
    orchestration purpose and degrading every cross-µservice carry path
    (file attach p99 would inflate by one workflow-engine hop).
purpose: |
  Define `drive`, `mail`, `messenger`, `calendar`, `recordings` as
  CROSS-CUTTING CARRIERS (not app-tier µservices) and permit direct gRPC
  egress from any oyatie app-tier µservice to these five carrier namespaces
  as a defined exemption to the Workflow+Ontology adapter rule
  (`feedback_workflow_objectgraph_adapter_layer`). All other cross-µservice
  flows continue to route through workflow-engine (orchestration) or Ontology
  (entity reads/writes). Mirrors AWS S3 / Google Cloud Storage / Anthropic
  Files API: the storage / mail / messaging carrier is a substrate every
  product binds to directly because the carrier IS the inter-product seam.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0140: Cross-cutting carriers — adapter-rule exemption

## Status

Accepted — 2026-05-18.

## Date

2026-05-18.

## Context

The Workflow+Ontology adapter rule (`feedback_workflow_objectgraph_adapter_layer`,
captured in user feedback 2026-05-15 and codified in ADR-0131, ADR-0132,
ADR-0135 and ADR-0136) states: **inter-product flows MUST traverse Workflow
(orchestration) or Ontology (entity reads/writes); product µservices MUST NOT
import or call sibling product µservices directly.**

This rule is enforced by:

- LEAN-A1 (dependency-direction): no cross-product crate imports.
- LEAN-A2 (cross-product-refusal): no cross-product Rust imports.
- per-µservice PRD §Bounded-Contexts cross-product clause.
- µservice NetworkPolicy default-deny posture.

The 2026-05-17 integration review (INT-001, INT-002, INT-003) found three
classes of egress in the substrate that appear, at first read, to violate
the rule:

1. **INT-001 — `network` → `ats`** (jobs-handoff). Resolved by routing the
   jobs-handoff event through workflow-engine. ATS is a discrete app-tier
   µservice and the workflow-engine route is the right answer.
2. **INT-002 — `meet` → `recordings`** (post-meeting recording handoff). The
   orchestration leg (when to ingest, retention-policy-binding,
   legal-hold-attachment) is a workflow-engine concern; the binary-payload
   pull (recordings fetching the muxed media from meet's S3 bucket) is a
   carrier concern.
3. **INT-003 — systemic** — many µservices egress directly to `drive`, `mail`,
   `messenger`, `calendar`, `recordings` for attachment / share / notify /
   bind / store flows. Examples:
   - `social` → `drive` for post-attachment upload
   - `sheets` → `mail` for share-by-email
   - `tasks` → `calendar` for due-date binding
   - `forms` → `messenger` for response-channel notification
   - `meet` → `recordings` for post-meeting blob persistence
   - `notes` → `drive` for attachment storage

INT-003 is a topology question, not a per-µservice fix. If we treat every
direct egress to one of these five µservices as a violation, then:

- workflow-engine becomes a chokepoint for every file-share, calendar-invite,
  channel-mention, recording-store, and email-share across the product
  portfolio. That defeats its orchestration purpose.
- Carrier latency inflates by one workflow-engine hop on every binary-payload
  carry path. File attach p99 inflates by ~30-80ms per hop.
- The Workflow+Ontology adapter-rule becomes unenforceable because reviewers
  must reason about which flows are orchestration (rule applies) vs. which
  are carry (rule cannot reasonably apply).

The reference hyperscaler model resolves this cleanly: AWS S3 is a substrate
that every AWS service binds to directly; Google Cloud Storage is the same;
Anthropic's Files API is the same; Slack's file-upload API is the same.
The storage / mail / messaging carrier IS the inter-product seam and
products bind to it directly. Workflow / orchestration sits ABOVE the
carriers and coordinates them, not BENEATH them as a relay.

## Decision

oyatie defines **CROSS-CUTTING CARRIERS** as a distinct µservice class with
five charter members and a defined exemption to the Workflow+Ontology
adapter rule:

| Carrier µservice | Carries on behalf of | Payload shape |
|---|---|---|
| `drive` | every µservice that attaches a file | binary blob + content_hash + tenant-DEK envelope |
| `mail` | every µservice that share-by-emails | RFC 5322 message + S/MIME envelope |
| `messenger` | every µservice that channel-mentions or notify-via-DMs | typed message + reaction + read-receipt |
| `calendar` | every µservice that binds a time-slot, due-date, RSVP, or invite | iCalendar VEVENT + ATTENDEE + retention bound |
| `recordings` | every µservice that persists a long-running media or audio stream | media manifest + S3 blob ref + retention floor |

### Exemption rule

Direct gRPC egress (or direct HTTPS, where the carrier exposes REST) from any
app-tier µservice to one of the five carrier namespaces is PERMITTED.
NetworkPolicies in `microservices/<ms>/iac/helm/<ms>/templates/networkpolicy.yaml`
MAY include an egress allow for any subset of the five carrier namespaces
when the µservice's PRD declares a carry concern for that carrier.

### Scope guardrail — what is NOT a carrier

The exemption is closed-set. A µservice qualifies as a cross-cutting carrier
ONLY if it satisfies ALL of:

1. It is named in the carrier list above (charter set; additions require an
   amending ADR).
2. It is consumed by ≥ 3 distinct app-tier µservices for a carry concern
   (attach / share / notify / bind / store), not for an orchestration
   concern (when-to-do, who-approves, what-policy-applies).
3. It carries data, not decisions. Decision-flow (workflow state machines,
   approval-routing, eligibility-evaluation) MUST continue to traverse
   workflow-engine.

Notably NOT carriers (and therefore still subject to the full adapter rule):

- `ats` (Tier-G applicant tracking system): an app-tier µservice; jobs-handoff
  is decision-flow, not data-carry. Route through workflow-engine. (INT-001.)
- `foundry` (single µservice per ADR-0136): a substrate, not a carrier;
  inference invocation traverses its public RPC contract directly, but the
  µservice itself is governed by ADR-0136 isolation, not by this ADR.
- `social`, `network`, `sheets`, `tasks`, `notes`, `forms`, `meet`, `community`,
  `shorts`, `events`, `news`, `polls`, `wiki`, `slides`, `flows`: app-tier
  product µservices. They MUST route inter-product flows through workflow-engine
  or Ontology except where this ADR's carrier exemption applies.

### Orchestration carve-out

Even within the carrier set, the orchestration leg of a multi-step handoff
MUST traverse workflow-engine. Examples:

- `meet` → `recordings` post-meeting persistence: the `meet.meeting.v1.ended`
  → `recordings.ingest.v1.requested` orchestration MUST flow through
  workflow-engine; the carrier exemption permits `recordings` to subsequently
  pull the binary blob from meet's S3 bucket via signed URL.
- `mail` → `drive` attachment-store: the routing decision (which drive shard,
  which retention policy, which legal-hold mask) is a workflow-engine concern
  when the tenant has authored a workflow over it; the carry itself is
  carrier-to-carrier direct.

The default carry path is direct; the orchestration overlay is workflow-engine.
A µservice that has no workflow attached to the carry concern flows direct.
A µservice whose tenant has attached a workflow (legal-hold, four-eyes
approval, retention-rebind) flows through workflow-engine for the trigger
leg and direct for the binary leg.

## Alternatives Considered

### (a) Route everything through workflow-engine (REJECTED)

Pure adapter-rule enforcement: every cross-µservice egress, including
attachment-upload to `drive`, share-by-email to `mail`, channel-mention to
`messenger`, calendar-bind to `calendar`, recording-store to `recordings`,
flows through workflow-engine as a typed event with the carrier as a
subscriber.

Rejected because:

- workflow-engine becomes a critical-path chokepoint for every file-share /
  share-by-email / channel-mention / calendar-bind / recording-store in
  the portfolio. Steady-state event-bus throughput requirement inflates by
  ~10× (every binary-payload carry adds a hop).
- Binary-payload carry through an event-bus inverts the carrier-of-record
  pattern; the bus would need to either ship the binary inline (memory
  exhaustion on large files) or ship a reference (which is just the direct
  carrier path with extra latency).
- Defeats the carrier's purpose. AWS S3, Google Cloud Storage, Anthropic
  Files API, and Slack file-upload all expose direct binding; the industry
  norm is direct carrier binding, not through-a-bus relay.

### (b) Route through Ontology entity binding only (REJECTED)

Treat every carry as an Ontology entity write: `Attachment{blob_ref, ...}` is
an Ontology object, every µservice that needs to attach a file writes the
Ontology entity and the carrier reads from Ontology.

Rejected because:

- Ontology is read-optimised (Palantir-class object graph; cf.
  feedback_glossary_ontology_not_object_graph). It doesn't carry binary
  payloads efficiently and was never sized for object-graph-as-CDN
  workloads.
- Ontology writes are typed and authorised through Cedar; binary-payload
  carry has a different authority model (signed-URL grants with explicit
  expiry).
- Ontology pollution: every transient attachment / channel-mention / RSVP
  becomes an Ontology entity, inflating the graph by 100-1000× and
  destroying query latency.

### (c) Define carriers explicitly and permit direct gRPC to them (ACCEPTED)

Charter five carriers, allow direct egress to them, require everything else
to flow through workflow-engine or Ontology. Clean separation between
orchestration (workflow-engine), entity reads/writes (Ontology), and carry
(drive / mail / messenger / calendar / recordings).

Accepted because:

- Matches the hyperscaler reference model (AWS / Google / Microsoft / Anthropic).
- Preserves workflow-engine's orchestration purpose without forcing it onto
  every binary-payload hop.
- Provides a closed, reviewable exemption surface (five carriers; not
  open-ended).
- Allows the Workflow+Ontology adapter rule to remain unambiguously enforced
  for the non-carrier surface.

## Consequences

### Positive

1. **Workflow+Ontology adapter rule scope precisely defined.** No more
   gray-zone networkpolicies. Reviewers (human + agentic) can determine
   compliance by reading the carrier list against the egress list.
2. **workflow-engine throughput envelope preserved.** Steady-state event
   volume is bounded by orchestration concerns, not by every binary-payload
   carry. Capacity model for workflow-engine remains tractable.
3. **Carry-path latency preserved.** No mandatory workflow-engine hop on
   file-share / share-by-email / channel-mention / calendar-bind /
   recording-store flows. p99 for the carry leg stays bounded by the
   carrier's own SLO.
4. **Industry alignment.** Matches AWS / Google / Microsoft / Anthropic
   reference shape; reviewers familiar with hyperscaler conventions read
   the topology immediately.
5. **Stronger SLA obligations on carriers are made explicit.** The five
   carriers MUST publish per-µservice SLAs because they're consumed
   directly by every product. Carrier outage = portfolio-wide outage; the
   SLO authority for each carrier escalates accordingly.

### Negative / cost

1. **LEAN-A2 architecture-boundaries validator needs an exemption list.**
   The Cedar policy under `microservices/governance/policy/` and the Rust
   validator under `crates/oya-check-architecture-boundaries/` (when it
   exists; see ADR §Follow-ups) must read a canonical carrier-list spec at
   `/specs/cross-cutting-carriers.json` and exempt direct egress to those
   five namespaces.
2. **Carrier µservices carry stronger availability obligations.** Because
   every app-tier µservice may egress directly, a carrier outage cascades
   across the portfolio. Carriers MUST target 99.99 % monthly minimum;
   they MUST shard horizontally; they MUST publish capacity envelopes
   independent of any single consumer.
3. **Charter is closed-set with high admission cost.** Adding a sixth
   carrier requires an amending ADR with a quorum from
   council-architecture + council-privacy + ops-security. No carrier may
   be silently added by a µservice author.
4. **Audit-chain emission is unchanged.** Direct carrier egress STILL
   emits audit-chain records for the carry event. The exemption is a
   networkpolicy / adapter-rule exemption only, NOT an audit-trail
   exemption.

### Risk / mitigation

| Risk | Mitigation |
|---|---|
| Reviewers mistake an app-tier µservice for a carrier and admit a direct egress | Carrier list is canonical at `/specs/cross-cutting-carriers.json` (Slice B); LEAN-A2 reads from that spec; admission gate refuses unmatched egress |
| Carrier scope creep: future µservice authors claim "we're a carrier" to bypass the adapter rule | Amending-ADR requirement + ≥3-consumer test + carries-data-not-decisions test enforced in the admission gate |
| Carrier becomes a critical-path bottleneck if its SLO degrades | Carriers MUST publish OpenSLO targets ≥ 99.99 % monthly per ADR-0139; agentic SLO-gated promotion enforces |

## References

- feedback_workflow_objectgraph_adapter_layer (the rule being scoped).
- feedback_canonical_base_localization (ADR-0064; carrier shape is a
  canonical base + per-pack localization overlay).
- feedback_no_silent_regression (carrier-list changes require amending ADR).
- ADR-0011 (cross-microservice contract registry; carrier contracts MUST
  be registered).
- ADR-0064 (canonical-base + localization; carrier µservices ship per-pack
  localization overlays).
- ADR-0123 (authority cohesion; each carrier carries its own authority root).
- ADR-0131 (per-microservice flat layout; carriers ship under
  `microservices/<carrier>/` like any other µservice).
- ADR-0132 (suite-and-bundle dissolution; carriers are NOT a suite; each
  is a single µservice).
- ADR-0135 (super-app expansion into 8 µservices; messenger / mail
  / calendar emerged as carriers from the dissolution).
- ADR-0136 (Foundry as single µservice; foundry is a substrate, not a
  carrier — distinct concept).
- ADR-0139 (agentic SLO-gated promotion; carriers MUST clear the elevated
  SLO bar).
- AWS S3 product architecture (carrier-of-record pattern).
- Google Cloud Storage object model (carrier-of-record pattern).
- Anthropic Files API design (carrier-of-record pattern).
- Slack file-upload API (carrier-of-record pattern with per-team RBAC).

## Cross-links from prior ADRs

This ADR refines the Workflow+Ontology adapter rule scope established in
ADR-0131, ADR-0132, ADR-0135, ADR-0136. Authors of those ADRs SHOULD add a
"see also ADR-0140" reference in a subsequent housekeeping IP. No content in
those ADRs is invalidated; ADR-0140 adds the carrier exemption that was
implicit in the hyperscaler-shape benchmark but not previously written down.

## Follow-ups

1. **Spec file**: author `/specs/cross-cutting-carriers.json` declaring the
   five-carrier charter, machine-readable for LEAN-A2 consumption. (Slice B.)
2. **Helm helper**: add `oya.networkPolicy.allowEgressToCarriers` to
   `microservices/governance/iac/helm/_oya-helpers/templates/_helpers.tpl`
   emitting the five carrier-namespace egresses; refactor existing
   networkpolicy.yaml templates to use it. (This ADR's accompanying fix
   includes the helper; existing µservices are refactored in follow-up IPs.)
3. **Validator update**: when `crates/oya-check-architecture-boundaries/`
   (or its successor) is built, it MUST consult
   `/specs/cross-cutting-carriers.json` and exempt direct egress to those
   five namespaces from the cross-product-refusal rule. Until then, the ADR
   itself is the canonical statement and tooling lags.
4. **PRD audit**: confirm each of the five carrier µservice PRDs declares
   the carrier role explicitly under §Bounded-Contexts §Tier classification.
   File follow-up IPs for any carrier whose PRD does not yet declare it.

## Numbering note

ADR-0140 is the next free slot in the oyatie ADR sequence following
ADR-0139 (agentic SLO-gated promotion, 2026-05-17). ADR-0131 has two
filenames in the repo (`-per-microservice-flat-layout.md` and
`-community-social-expansion-planning-contract.md`); per
ADR-0135's renumbering precedent, this duplication is a separate
housekeeping concern and does not affect this ADR's numbering.
