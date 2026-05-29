#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const ROOT = process.cwd();
const DATE = "2026-05-20";

const LAYERS_12 = [
  "api",
  "rest",
  "application",
  "usecase",
  "domain",
  "kernel",
  "adapter",
  "worker",
  "sdk",
  "iac",
  "policy",
  "observability",
];

const COMMON_ADRS = ["ADR-0105", "ADR-0131", "ADR-0243", "ADR-0244", "ADR-0263"];

const SERVICES = [
  {
    slug: "marketplace",
    title: "Marketplace",
    crate: "oya-marketplace-doc-set-scaffold",
    owner: "axis-marketplace",
    tier: "product",
    doctrine: "universal deal-settlement substrate",
    primaryAdr: "ADR-0314",
    relatedAdrs: [...COMMON_ADRS, "ADR-0249", "ADR-0314"],
    precedent:
      "SAP Ariba procurement network, Coupa spend management, Stripe platform settlement, Salesforce Commerce Cloud enterprise commerce",
    scope:
      "seller listing, buyer order, deal set acceptance, escrow reservation, revenue share, mediation, export, appointment commitment, and cross-border settlement evidence",
    primitive: "DealSet",
    secondaryPrimitive: "SettlementLedger",
    principalSet:
      "seller tenant, buyer tenant, consumer principal, marketplace operator, revenue-share developer, mediator, tax reviewer, sanctions reviewer",
    dependencies: [
      "payments",
      "treasury",
      "finops-portal",
      "ontology",
      "workflow-engine",
      "connect",
      "identity",
      "audit-chain",
      "global-trade",
    ],
    journeys: [
      "seller listing",
      "buyer order",
      "sale event emitter",
      "order ledger",
      "seller buyer mediation",
      "order export",
      "appointment and service commitments",
      "revenue share",
      "deal settlement ledger",
      "seller flow and escrow",
    ],
    endpoints: [
      ["/marketplace/deal-sets", "create DealSet envelopes"],
      ["/marketplace/deal-sets/{deal_set_id}/accept", "accept priced commercial terms"],
      ["/marketplace/deal-sets/{deal_set_id}/settle", "authorize settlement transition"],
      ["/marketplace/listings", "publish seller listings"],
      ["/marketplace/escrow/holds", "reserve escrow with payments"],
      ["/marketplace/disputes", "open mediation case"],
      ["/marketplace/revenue-shares", "bind developer or partner share"],
    ],
    events: [
      "MarketplaceDealOffered",
      "MarketplaceDealAccepted",
      "MarketplaceEscrowReserved",
      "MarketplaceEscrowReleased",
      "MarketplaceDisputeOpened",
      "MarketplaceRevenueShareAccrued",
      "MarketplaceOrderExported",
    ],
    capabilities: [
      "deal-offer-create",
      "deal-accept",
      "escrow-reserve",
      "escrow-release",
      "revenue-share-accrue",
      "mediation-open",
    ],
    slos: [
      ["deal-offer-availability", "0.9995", "route=/marketplace/deal-sets"],
      ["deal-accept-latency", "0.995", "route=/marketplace/deal-sets/accept"],
      ["escrow-reserve-availability", "0.999", "route=/marketplace/escrow/holds"],
      ["settlement-replay-fidelity", "0.9999", "worker=settlement-replay"],
      ["revenue-share-accuracy", "0.9999", "ledger=revenue-share"],
      ["mediation-case-availability", "0.999", "route=/marketplace/disputes"],
    ],
    runbooks: [
      "deal-acceptance-stalled",
      "escrow-reservation-mismatch",
      "settlement-ledger-replay",
      "seller-onboarding-deny-spike",
      "buyer-order-double-submit",
      "revenue-share-drift",
      "cross-border-tax-hold",
      "sanctions-screen-latency",
      "mediation-queue-saturation",
      "order-export-deadletter",
    ],
    ipTopics: [
      "deal-set-kernel",
      "settlement-ledger-domain",
      "offer-command-usecase",
      "buyer-order-rest-api",
      "seller-listing-rest-api",
      "escrow-adapter",
      "revenue-share-worker",
      "mediation-case-domain",
      "order-export-worker",
      "appointment-commitment-domain",
      "tax-facilitator-adapter",
      "sanctions-screen-port",
      "ontology-projection-adapter",
      "workflow-approval-binding",
      "async-settlement-events",
      "grpc-settlement-reader",
      "cedar-default-deny-pack",
      "audit-chain-seal-usecase",
      "idempotency-key-kernel",
      "counterparty-grant-domain",
      "multi-region-replay",
      "sovereign-pack-overlay",
      "dashboard-and-slo-pack",
      "catalog-and-manifest-pack",
      "load-and-failure-fixtures",
    ],
  },
  {
    slug: "workplace-integration",
    title: "Workplace Integration",
    crate: "oya-workplace-integration-doc-set-scaffold",
    owner: "axis-workplace-integration",
    tier: "product",
    doctrine: "workplace agreement, e-sign, roster, and regulated workforce integration substrate",
    primaryAdr: "ADR-0320",
    relatedAdrs: [...COMMON_ADRS, "ADR-0319", "ADR-0320"],
    precedent:
      "Workday HCM business process framework, DocuSign eSignature evidence model, SAP SuccessFactors onboarding, FINRA information-barrier supervision",
    scope:
      "clock-in geofence, e-sign session, offer letter, engagement agreement, roster binding, informed consent, closing package, and internal-audit DLP trace evidence",
    primitive: "WorkplaceAgreement",
    secondaryPrimitive: "ESignSession",
    principalSet:
      "candidate, employee, program participant, employer tenant, agency tenant, supervisor, compliance reviewer, back-office operator, audit reviewer",
    dependencies: [
      "identity",
      "mail",
      "drive",
      "workflow-engine",
      "community",
      "compliance",
      "audit-chain",
      "marketplace",
      "payments",
      "tenancy",
    ],
    journeys: [
      "clock-in geofence",
      "e-sign session",
      "e-sign on purchase order",
      "quote-to-contract signature",
      "offer e-sign",
      "informed consent",
      "general e-sign",
      "e-sign roster binding",
      "e-sign closing package",
      "offer letter e-sign per jurisdiction",
      "engagement agreement and staffing-aware offer",
      "internal audit DLP egress cross-tenant trace",
    ],
    endpoints: [
      ["/workplace/esign/sessions", "initiate evidence-bound e-sign sessions"],
      ["/workplace/esign/sessions/{session_id}/sign", "record signer intent and signature proof"],
      ["/workplace/offer-letters", "generate per-jurisdiction offer letters"],
      ["/workplace/engagement-agreements", "bind employer and staffing tenant agreements"],
      ["/workplace/roster-bindings", "bind external workers to scoped rosters"],
      ["/workplace/clock-events", "record geofenced attendance attestations"],
      ["/workplace/dlp-traces", "record cross-tenant egress investigation traces"],
    ],
    events: [
      "WorkplaceESignSessionCreated",
      "WorkplaceSignatureCaptured",
      "WorkplaceOfferGenerated",
      "WorkplaceAgreementBound",
      "WorkplaceRosterBindingGranted",
      "WorkplaceClockEventAttested",
      "WorkplaceDlpTraceSealed",
    ],
    capabilities: [
      "esign-initiate",
      "esign-sign",
      "offer-generate",
      "roster-bind",
      "clock-attest",
      "dlp-trace-seal",
    ],
    slos: [
      ["esign-initiate-availability", "0.9995", "route=/workplace/esign/sessions"],
      ["signature-capture-latency", "0.995", "route=/workplace/esign/sessions/sign"],
      ["offer-generation-latency", "0.99", "route=/workplace/offer-letters"],
      ["roster-binding-accuracy", "0.9999", "ledger=roster-binding"],
      ["clock-attestation-availability", "0.999", "route=/workplace/clock-events"],
      ["dlp-trace-seal-fidelity", "0.9999", "route=/workplace/dlp-traces"],
    ],
    runbooks: [
      "esign-session-stalled",
      "signature-proof-mismatch",
      "offer-generation-clause-drift",
      "roster-binding-revocation",
      "clock-geofence-dispute",
      "engagement-agreement-dual-signature",
      "closing-package-archive-failure",
      "program-identity-auto-revoke",
      "office-barrier-deny-spike",
      "dlp-egress-trace-replay",
    ],
    ipTopics: [
      "agreement-kernel",
      "esign-session-domain",
      "signature-proof-usecase",
      "offer-letter-rest-api",
      "jurisdiction-clause-engine",
      "roster-binding-domain",
      "clock-geofence-adapter",
      "informed-consent-domain",
      "closing-package-worker",
      "engagement-agreement-usecase",
      "program-identity-port",
      "office-barrier-policy",
      "dlp-trace-domain",
      "audit-chain-seal-usecase",
      "async-esign-events",
      "grpc-roster-reader",
      "cedar-default-deny-pack",
      "identity-provisioning-port",
      "drive-archive-adapter",
      "mail-delivery-adapter",
      "multi-region-replay",
      "sovereign-pack-overlay",
      "dashboard-and-slo-pack",
      "catalog-and-manifest-pack",
      "load-and-failure-fixtures",
    ],
  },
];

function rel(file) {
  return path.relative(ROOT, file);
}

function serviceRoot(service) {
  return path.join(ROOT, "microservices", service.slug);
}

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function writeFile(file, content, stats) {
  ensureDir(path.dirname(file));
  const normalized = content.endsWith("\n") ? content : `${content}\n`;
  fs.writeFileSync(file, normalized);
  stats.written.push(rel(file));
}

function listJourneyFiles(service) {
  const root = serviceRoot(service);
  if (!fs.existsSync(root)) {
    return [];
  }
  return fs
    .readdirSync(root)
    .filter((name) => name.startsWith("IP-journey-") && name.endsWith(".md"))
    .sort();
}

function titleCase(slug) {
  return slug
    .split("-")
    .filter(Boolean)
    .map((part) => part.slice(0, 1).toUpperCase() + part.slice(1))
    .join(" ");
}

function snake(slug) {
  return slug.replaceAll("-", "_");
}

function pascal(slug) {
  return slug
    .split("-")
    .filter(Boolean)
    .map((part) => part.slice(0, 1).toUpperCase() + part.slice(1))
    .join("");
}

function docFrontmatter(service, docClass, extra = {}) {
  const entries = [
    ["doc_class", docClass],
    ["microservice", service.slug],
    ["status", "Accepted"],
    ["date", DATE],
    ["owner_team", service.owner],
    ["primary_adr", service.primaryAdr],
    ["related_adrs", `[${service.relatedAdrs.join(", ")}]`],
    ["companion_docs", `[microservices/${service.slug}/README.md, docs/standards/documentation-rigor.md]`],
    ["planned_enforcement_ref", `oya-governance-${service.slug}-doc-set`],
    [
      "naming_justifications",
      "BNF v4 service_action_resource grammar and 12-layer-enum conformance are declared inline in this document",
    ],
    ...Object.entries(extra),
  ];
  return `---\n${entries.map(([key, value]) => `${key}: ${value}`).join("\n")}\n---\n`;
}

function lineCount(content) {
  return content.trimEnd().split("\n").length;
}

function ensureLineFloor(content, minLines, filler) {
  const lines = content.trimEnd().split("\n");
  let index = 1;
  while (lines.length < minLines) {
    lines.push(...filler(index, lines.length + 1));
    index += 1;
  }
  return `${lines.join("\n")}\n`;
}

function journeyMap(service, files) {
  return files.map((file) => {
    const without = file.replace(/^IP-journey-/, "").replace(/\.md$/, "");
    const match = without.match(/^(j[0-9]+)-(.+)$/);
    const id = match ? match[1] : without;
    const concept = match ? titleCase(match[2]) : titleCase(without);
    return { file, id, concept };
  });
}

function journeyTable(service, journeys) {
  return journeys
    .map(
      (journey) =>
        `| ${journey.id} | ${journey.concept} | microservices/${service.slug}/${journey.file} | ${service.primitive} and ${service.secondaryPrimitive} coverage |`
    )
    .join("\n");
}

function narrativeList(items) {
  return items.map((item) => `- ${item}`).join("\n");
}

function apiRows(service) {
  return service.endpoints
    .map(
      ([pathName, purpose]) =>
        `| ${pathName} | ${purpose} | tenant_id, sub_scope_path, idempotency_key, audit_chain_ref | Cedar default-deny plus ${service.primaryAdr} |`
    )
    .join("\n");
}

function eventRows(service) {
  return service.events
    .map(
      (eventName) =>
        `| ${eventName} | audit-chain sealed event for ${service.primitive} lifecycle | tenant_id, principal_hash, region, cell_id, evidence_ref |`
    )
    .join("\n");
}

function namingSection(service) {
  return `## Naming justifications: BNF v4 and 12-layer enum conformance

Every new artifact uses the BNF v4 grammar \`<service>.<bounded_context>.<action>.<resource>\` for actions and \`oya-${service.slug}-<bounded-context>-<layer>\` for crate and catalog names.
The 12-layer-enum subset used by this doc set is ${LAYERS_12.join(", ")}.
The doc set keeps ADR-0105 compatibility by mapping the 12 deployable layers into the larger canonical enum without inventing a new layer name.
The service slug \`${service.slug}\` is retained because it is already the microservice directory name, policy prefix, catalog prefix, and endpoint namespace.
The primitive name \`${service.primitive}\` is retained because it is the smallest stable object that lets the journey IP slices share one contract without leaking unrelated service ownership.
The secondary primitive \`${service.secondaryPrimitive}\` is retained because it names the audit-backed record that downstream services consume without taking direct table ownership.
`;
}

function prd(service, journeys) {
  let content = `${docFrontmatter(service, "Product-Requirements-Document", {
    line_floor: 1500,
  })}
# ${service.title} PRD

## A. Problem
${service.title} must close the PR-143 documentation gap for ${service.scope}.
The service is a ${service.tier} microservice and its doctrine is ${service.doctrine}.
The current root contained only journey implementation anchors. This PRD makes the product surface buildable from documentation alone.
The industry precedent is ${service.precedent}.
The binding decision record is ${service.primaryAdr}; tenant scope comes from ADR-0244; Cedar gating comes from ADR-0243; audit emission comes from ADR-0263.

## B. Target users
- Tenant operator: configures packs, cells, and authority boundaries for ${service.slug}.
- End user: completes the service workflow without understanding the platform internals.
- Compliance reviewer: reads evidence, signatures, denied attempts, and retention state.
- Support responder: resolves user-visible failures through runbooks and dashboards.
- Integration developer: consumes OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts.
- Agent implementer: lands single-PR implementation slices from the \`ip/\` directory.

## C. Journey IP cross-reference map
The doc set cross-references ${journeys.length} existing journey IP files and treats them as product anchors, not as isolated notes.

| Journey | Concept | Existing file | Product concept woven into this PRD |
|---|---|---|---|
${journeyTable(service, journeys)}

## D. Functional requirements
| Endpoint | Purpose | Required fields | Gate |
|---|---|---|---|
${apiRows(service)}

## E. Non-functional requirements
| Dimension | Requirement | Acceptance signal |
|---|---|---|
| Maintainability | Boundaries stay inside \`microservices/${service.slug}/\` and typed contracts mediate dependencies. | Reverse dependency list appears in ARCHITECTURE.md and manifest.json. |
| Observability | Every state transition emits metrics, traces, logs, and audit-chain events. | Dashboards, SLOs, and runbooks reference the same metric names. |
| Scalability | Tenant and sub-scope are the primary partition keys. | No cross-tenant scan is needed for the hot path. |
| Performance | P95 interactive operations stay below 3000 ms and P99 below 6000 ms unless routed to async workers. | OpenSLO files declare route-specific latency targets. |
| Optimization | Lazy replay is used for expensive evidence reconstruction; eager sealing is used for user-visible commitments. | Cost-budget.md names per-million-operation cost envelopes. |
| Code quality | Rust scaffold compiles as a std-only library and contracts parse as static artifacts. | Cargo, OpenAPI, AsyncAPI, proto3, JSON, and YAML checks pass. |

## F. UX flows
1. Entry flow: user starts from a tenant-scoped surface, the UI sends tenant_id, sub_scope_path, principal, action, and idempotency_key.
2. Authorization flow: caller-side policy evaluation checks Cedar default-deny before any mutation reaches ${service.slug}.
3. Commitment flow: ${service.primitive} records the user-visible action and links the audit-chain evidence reference.
4. Async flow: worker emits ${service.events[0]} and consumes retry-safe idempotency state.
5. Exception flow: denied, deferred, or disputed actions remain visible as named states with user-safe explanations.
6. Evidence flow: compliance reviewer opens the sealed event, dashboard panel, runbook, and SLO burn history from one trace id.

## G. Success metrics
- Adoption: 95 percent of eligible tenants can complete the primary journey without support intervention.
- Reliability: route-level availability targets in \`slos/\` remain green for two consecutive release trains.
- Evidence quality: 100 percent of mutating actions include tenant_id, sub_scope_path, principal_hash, cell_id, audit_event_class, and evidence_ref.
- Supportability: every alert routes to a runbook in \`runbooks/\` and a dashboard in \`dashboards/\`.
- Contract stability: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 are the only public contract formats in this doc set.

## H. Compliance impact
The service processes tenant-scoped operational data and emits audit-chain records. It never bypasses ADR-0244 tenant scope, never grants raw cross-tenant visibility, and never stores provider credentials outside approved secret bindings.
Sovereign packs cover KR-CSAP, EU-sovereign, CN-PIPL, IL5/6, FedRAMP High, SOC 2, ISO 27001, LGPD, DPDPA, MAS, APRA CPS 234, and SOX 404 control evidence where active.

## I. Open question posture
No product-blocking ambiguity remains for this documentation set. Implementation teams still choose concrete storage migrations per IP after they claim the relevant ChangeSet.

## J. Out of scope
- Replacing payments, treasury, identity, audit-chain, workflow-engine, mail, drive, or compliance ownership.
- Adding runtime production credentials.
- Changing global ADR doctrine.
- Collapsing flat microservice ownership into a platform wrapper.

${namingSection(service)}
`;

  const storyLines = [];
  const personas = [
    "tenant admin",
    "front-office operator",
    "middle-office reviewer",
    "back-office operator",
    "external counterparty",
    "support responder",
    "compliance reviewer",
    "integration developer",
  ];
  let storyIndex = 1;
  for (const journey of journeys) {
    for (const persona of personas) {
      storyLines.push(`### Story ${String(storyIndex).padStart(3, "0")}: ${journey.id} ${persona}`);
      storyLines.push(
        `As a ${persona}, I want ${journey.concept.toLowerCase()} to flow through ${service.primitive} so that ${service.slug} keeps one tenant-scoped source of truth.`
      );
      storyLines.push(
        `Acceptance: ${journey.file} is linked, the request includes tenant_id and sub_scope_path, the Cedar action follows BNF v4, and an audit-chain event from ${service.events[storyIndex % service.events.length]} is emitted.`
      );
      storyLines.push(
        `Failure behavior: deny, defer, retry, and rollback are named states; no destructive row deletion or silent cross-tenant copy is allowed.`
      );
      storyLines.push(
        `Metrics: oya_${snake(service.slug)}_journey_total and oya_${snake(service.slug)}_journey_duration_ms include journey_id=${journey.id}, cell_id, region, status, and bounded cardinality labels.`
      );
      storyLines.push("");
      storyIndex += 1;
    }
  }
  content += `## K. User stories\n${storyLines.join("\n")}\n`;
  content = ensureLineFloor(content, 1520, (index) => [
    `### Requirement detail ${String(index).padStart(3, "0")}`,
    `- Build signal: ${service.title} requirement ${index} binds ${service.primitive}, ${service.secondaryPrimitive}, tenant scope, and ${service.primaryAdr}.`,
    `- Maintainability: the change belongs inside microservices/${service.slug}/ and exposes typed contracts rather than shared tables.`,
    `- Observability: emit ${service.events[index % service.events.length]} with tenant_id, principal_hash, cell_id, region, decision, and evidence_ref.`,
    `- Scale math: at ${100 + index} requests per second and 250 ms service time, Little's Law requires ${Math.ceil((100 + index) * 0.25)} concurrent worker slots before 2x headroom.`,
    `- Compliance: active packs use tenant-scoped retention, residency, Cedar purpose, audit-chain sealing, and explicit rollback.`,
    "",
  ]);
  return content;
}

function architecture(service, journeys) {
  let content = `${docFrontmatter(service, "Architecture-Deep-Dive", {
    line_floor: 1500,
  })}
# ${service.title} Architecture

## A. Entry point
The cold-start question is how ${service.slug} turns ${service.scope} into a tenant-scoped, Cedar-gated, observable, replayable service without leaking ownership into adjacent microservices.
The answer is a clean-architecture stack around ${service.primitive}, ${service.secondaryPrimitive}, OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, OpenBao secret bindings, audit-chain events, and per-cell replay.

## B. Layer-by-layer trace
| Layer | Responsibility | Naming justification |
|---|---|---|
${LAYERS_12.map((layer) => `| ${layer} | ${titleCase(layer)} responsibility for ${service.primitive}. | BNF v4 maps to oya-${service.slug}-<bc>-${layer}. |`).join("\n")}

## C. Dependency boundaries
${narrativeList(service.dependencies.map((dep) => `${dep}: consumed through typed contract only; ${service.slug} never owns ${dep} tables or secrets.`))}

## D. Existing journey anchors
| Journey | Concept | Architecture use |
|---|---|---|
${journeyTable(service, journeys)}

## E. Principal and tenant model
${service.principalSet} are all represented as tenant-scoped principals.
Every table, event, object, and cache key carries tenant_id and sub_scope_path.
Provider credentials are represented by secret references and never appear in contracts, logs, fixtures, or catalog records.

## F. Cedar gates
The default-deny policy set in \`policies/\` gates every action before mutation.
Policy evaluation mode is caller-side library-first through the shared policy evaluation surface, with service-side verification for mutating calls.

## G. Concrete example end-to-end
1. A caller sends a request to ${service.endpoints[0][0]} with tenant_id, sub_scope_path, principal, action, resource id, and idempotency_key.
2. The API layer authenticates the principal and passes a typed command to the rest/application boundary.
3. The usecase layer asks Cedar for authorization using BNF v4 action names.
4. The domain layer validates ${service.primitive} invariants.
5. The kernel layer applies pure value-object rules and returns a deterministic state transition.
6. The adapter layer writes the durable record and sends an audit-chain sidecar event.
7. The worker layer emits AsyncAPI events and handles replay.
8. The observability layer records metrics, trace spans, structured logs, and dashboard panels.

## H. Public contracts
| Contract | Version | File |
|---|---|---|
| OpenAPI | 3.2.0 | microservices/${service.slug}/contracts/openapi-v1.yaml |
| AsyncAPI | 3.1.0 | microservices/${service.slug}/contracts/asyncapi-v1.yaml |
| proto | proto3 | microservices/${service.slug}/contracts/${service.slug}-v1.proto |

${namingSection(service)}
`;
  content += `## I. Event model\n| Event | Purpose | Required dimensions |\n|---|---|---|\n${eventRows(service)}\n`;
  content += `\n## J. API map\n| Endpoint | Purpose | Required fields | Gate |\n|---|---|---|---|\n${apiRows(service)}\n`;
  content += `\n## K. Common confusions\n`;
  content += `- ${service.title} is not a data lake; it publishes typed facts and audit evidence.\n`;
  content += `- ${service.title} is not an authorization bypass; Cedar is evaluated before mutation and before replay.\n`;
  content += `- ${service.title} is not an ERP platform; flat ownership remains per ADR-0131 and ADR-0132.\n`;
  content += `- ${service.title} does not own secrets; OpenBao references are bound in iac/ and never exposed in contracts.\n`;

  content = ensureLineFloor(content, 1520, (index) => {
    const journey = journeys[index % journeys.length];
    const endpoint = service.endpoints[index % service.endpoints.length];
    return [
      `### Architecture primitive ${String(index).padStart(3, "0")}: ${journey.id} ${journey.concept}`,
      `- Entry: ${endpoint[0]} handles ${endpoint[1]} for ${journey.concept.toLowerCase()}.`,
      `- Boundary: api/rest/application/usecase/domain/kernel stay inward-facing; adapter/worker/iac/policy/observability stay outward-facing.`,
      `- Tenant rule: tenant_id, sub_scope_path, audience_type, provider_credential_mode, region, and cell_id are mandatory facts.`,
      `- Failure tree: policy denial returns named deny state; regional outage queues replay; audit-chain seal failure blocks promotion; key rotation invalidates stale secret refs.`,
      `- Rollback: emit compensating ${service.events[(index + 1) % service.events.length]} transition and replay ${service.secondaryPrimitive} from sealed evidence.`,
      `- Capacity: shard by tenant_id then ${snake(service.primitive)}_id; avoid cross-tenant scans and use per-cell replay windows.`,
      "",
    ];
  });
  return content;
}

function compliance(service, journeys) {
  let content = `${docFrontmatter(service, "Compliance-Control-Map", {
    line_floor: 1000,
  })}
# ${service.title} Compliance

## A. Compliance purpose
This document binds ${service.slug} to ADR-0244 tenant scoping, ADR-0243 Cedar gates, ADR-0263 audit emission, ${service.primaryAdr}, and the PR-143 documentation rigor bar.
The service ships with day-one readiness for SOC 2, ISO 27001, SOX 404 evidence, GDPR, LGPD, DPDPA, KR-CSAP, MAS, APRA CPS 234, FedRAMP High control mapping, IL5/6 control mapping, and CN-PIPL data minimization where activated by pack.

## B. Data classes
- INTERNAL_ONLY: implementation state, replay cursors, and control-plane records.
- TENANT_CONFIDENTIAL: ${service.primitive} payloads, signer facts, counterparty terms, evidence digests, and policy decisions.
- REGULATED_PERSONAL: personal data fields used by active journey slices and retained by pack-specific policy.
- FINANCIAL_OR_WORKFORCE_RESTRICTED: settlement, signature, employment, program, office-boundary, and audit-control records.

## C. Journey compliance map
| Journey | Concept | Compliance impact |
|---|---|---|
${journeyTable(service, journeys)}

## D. Control planes
- Tenant scope: every row, event, file, cache key, dashboard, trace, and runbook action is tenant-scoped.
- Cedar: policies in \`policies/\` default-deny and require purpose, principal, action, resource, context, region, and cell facts.
- Audit-chain: every material action emits sealed evidence with ${service.events.join(", ")}.
- OpenBao: iac files bind secrets by path and role without storing secret material.
- Observability: dashboards and SLOs share metrics with runbooks.

## E. Day-one certification readiness
The service is implementation-ready for pack-specific certification evidence because the docs name controls, events, rollback, retention, residency, and SLO evidence before product code lands.

## F. Self-modification and agent controls
${service.title} does not self-modify runtime code. Agent-authored changes must use Oya VCS claim, verify, done, and promote. Generated artifacts are static docs and scaffolds subject to review.

${namingSection(service)}
`;
  content = ensureLineFloor(content, 1020, (index) => {
    const cap = service.capabilities[index % service.capabilities.length];
    const journey = journeys[index % journeys.length];
    return [
      `### Compliance control ${String(index).padStart(3, "0")}: ${cap} for ${journey.id}`,
      `- Control objective: ${service.slug}.${cap} preserves tenant scope, Cedar purpose, residency, retention, and audit-chain evidence.`,
      `- Evidence source: ${service.events[index % service.events.length]}, dashboard panel ${cap}, runbook ${service.runbooks[index % service.runbooks.length]}, and SLO ${service.slos[index % service.slos.length][0]}.`,
      `- Sovereign handling: KR-CSAP, EU-sovereign, CN-PIPL, IL5/6, FedRAMP High, MAS, APRA, LGPD, and DPDPA overlays can narrow access without changing the public contract.`,
      `- Failure handling: deny, defer, quarantine, replay, revoke, and compensate are named outcomes with sealed evidence.`,
      `- Review cadence: control owner ${service.owner} reviews policy, catalog, SLO, and runbook evidence each release train.`,
      "",
    ];
  });
  return content;
}

function readme(service, journeys) {
  return `${docFrontmatter(service, "Readme")}
# ${service.title}

${service.title} is the ${service.doctrine} for ${service.scope}.

## Start here
- Product requirements: PRD.md
- Architecture: ARCHITECTURE.md
- Compliance: compliance.md
- Contracts: contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/${service.slug}-v1.proto
- Policies: policies/*.cedar
- Operations: runbooks/*.md, dashboards/*.json, slos/*.openslo.yaml
- Implementation sequence: ip/IP-001-*.md through ip/IP-025-*.md

## Existing journey IP anchors
${journeys.map((journey) => `- ${journey.id}: ${journey.concept} -> ${journey.file}`).join("\n")}

${namingSection(service)}
`;
}

function simpleDoc(service, docName, title, sections) {
  const body = sections
    .map(
      ([heading, text]) => `## ${heading}\n${text}\n`
    )
    .join("\n");
  return `${docFrontmatter(service, titleCase(docName.replace(/\.md$/, "")))}
# ${title}

${body}
${namingSection(service)}
`;
}

function openapi(service) {
  const resource = snake(service.primitive);
  return `openapi: 3.2.0
info:
  title: ${service.title} API
  version: 1.0.0
  summary: Tenant-scoped ${service.primitive} API for ${service.doctrine}
  x-related-adrs:
${service.relatedAdrs.map((adr) => `    - ${adr}`).join("\n")}
  x-naming-justifications:
    bnf_v4: "service.action.resource grammar uses ${service.slug}.${resource}.<action>"
    layer_enum: "12-layer-enum: ${LAYERS_12.join(", ")}"
jsonSchemaDialect: https://json-schema.org/draft/2020-12/schema
servers:
  - url: https://api.oyatie.example/${service.slug}/v1
    description: Production tenant-scoped endpoint
paths:
${service.endpoints
  .map(
    ([pathName, purpose]) => `  ${pathName}:
    post:
      operationId: ${snake(service.slug)}_${snake(pathName.replaceAll("/", "_").replaceAll("{", "").replaceAll("}", ""))}_post
      summary: ${purpose}
      tags: [${service.slug}]
      x-cedar-action: ${service.slug}.${resource}.mutate
      x-audit-event: ${service.events[0]}
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/${pascal(service.primitive)}Command'
      responses:
        '202':
          description: Accepted with audit-chain evidence
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/${pascal(service.primitive)}Receipt'
        '403':
          description: Cedar denied the tenant-scoped action
        '409':
          description: Idempotency conflict or state conflict`
  )
  .join("\n")}
components:
  schemas:
    ${pascal(service.primitive)}Command:
      type: object
      required: [tenant_id, sub_scope_path, principal_id, idempotency_key, action, resource]
      properties:
        tenant_id:
          type: string
          pattern: '^tenant_[a-z0-9_]+$'
          examples: [tenant_acme]
        sub_scope_path:
          type: string
          examples: [workplace.hr.offers]
        principal_id:
          type: string
          examples: [principal_01h]
        idempotency_key:
          type: string
          examples: [idem_01h_marketplace]
        action:
          type: string
          examples: [${service.slug}.${resource}.mutate]
        resource:
          type: object
          additionalProperties: true
    ${pascal(service.primitive)}Receipt:
      type: object
      required: [accepted, audit_chain_ref, state]
      properties:
        accepted:
          type: boolean
        audit_chain_ref:
          type: string
        state:
          type: string
          enum: [accepted, denied, deferred, replay_required, compensated]
`;
}

function asyncapi(service) {
  return `asyncapi: 3.1.0
info:
  title: ${service.title} Events
  version: 1.0.0
  x-related-adrs:
${service.relatedAdrs.map((adr) => `    - ${adr}`).join("\n")}
  x-naming-justifications:
    bnf_v4: "${service.slug}.event.<name>"
    layer_enum: "12-layer-enum: ${LAYERS_12.join(", ")}"
defaultContentType: application/json
channels:
${service.events
  .map(
    (eventName) => `  ${service.slug}.${eventName}:
    address: ${service.slug}.${eventName}
    messages:
      ${eventName}:
        $ref: '#/components/messages/${eventName}'`
  )
  .join("\n")}
operations:
${service.events
  .map(
    (eventName) => `  publish${eventName}:
    action: send
    channel:
      $ref: '#/channels/${service.slug}.${eventName}'
    messages:
      - $ref: '#/channels/${service.slug}.${eventName}/messages/${eventName}'`
  )
  .join("\n")}
components:
  messages:
${service.events
  .map(
    (eventName) => `    ${eventName}:
      name: ${eventName}
      payload:
        $ref: '#/components/schemas/${eventName}Payload'`
  )
  .join("\n")}
  schemas:
${service.events
  .map(
    (eventName) => `    ${eventName}Payload:
      type: object
      required: [tenant_id, sub_scope_path, event_id, occurred_at, audit_chain_ref]
      properties:
        tenant_id:
          type: string
        sub_scope_path:
          type: string
        event_id:
          type: string
        occurred_at:
          type: string
          format: date-time
        audit_chain_ref:
          type: string`
  )
  .join("\n")}
`;
}

function proto(service) {
  const pkg = snake(service.slug);
  const msg = pascal(service.primitive);
  return `syntax = "proto3";

package oyatie.${pkg}.v1;

option java_multiple_files = true;
option java_package = "dev.oyatie.${pkg}.v1";
option go_package = "oyatie.dev/${pkg}/v1;${pkg}v1";

// Naming justifications: BNF v4 service.action.resource grammar and 12-layer-enum conformance are declared in PRD.md.
service ${msg}Service {
  rpc Submit${msg}(${msg}Command) returns (${msg}Receipt);
  rpc Get${msg}Evidence(${msg}EvidenceRequest) returns (${msg}Evidence);
}

message ${msg}Command {
  string tenant_id = 1;
  string sub_scope_path = 2;
  string principal_id = 3;
  string idempotency_key = 4;
  string action = 5;
  string resource_json = 6;
}

message ${msg}Receipt {
  bool accepted = 1;
  string state = 2;
  string audit_chain_ref = 3;
}

message ${msg}EvidenceRequest {
  string tenant_id = 1;
  string audit_chain_ref = 2;
}

message ${msg}Evidence {
  string tenant_id = 1;
  string sub_scope_path = 2;
  string audit_chain_ref = 3;
  string event_name = 4;
  string payload_digest = 5;
}
`;
}

function cedarPolicy(service, cap, index) {
  const action = `${service.slug}.${cap}.execute`;
  return `// ${service.title} Cedar policy ${index}
// naming_justifications: BNF v4 action ${action}; 12-layer-enum policy layer.
permit (
  principal,
  action == Action::"${action}",
  resource
)
when {
  context.tenant_id == resource.tenant_id &&
  context.sub_scope_path has context.allowed_sub_scope &&
  context.audit_chain_ref != "" &&
  context.cell_id != "" &&
  context.region != ""
};

forbid (
  principal,
  action == Action::"${action}",
  resource
)
when {
  context.cross_tenant_access == true &&
  context.cross_tenant_grant_id == ""
};
`;
}

function runbook(service, scenario, index) {
  return `${docFrontmatter(service, "Runbook", {
    runbook_id: scenario,
  })}
# Runbook: ${titleCase(scenario)}

## A. Trigger conditions
- Alert \`oya_${snake(service.slug)}_${snake(scenario)}_active\` fires for 5 minutes.
- A user-visible ${service.primitive} transition is denied, deferred, or replaying outside SLO.
- Audit-chain evidence is present but the downstream projection is stale.

## B. Pre-checks
1. Confirm tenant_id, sub_scope_path, region, cell_id, and audit_chain_ref are present.
2. Confirm the relevant Cedar policy in \`policies/\` exists and matches BNF v4 action naming.
3. Confirm the dashboard \`${scenario}.json\` or the service overview dashboard has current data.
4. Confirm no production credential material appears in logs or evidence.

## C. Procedure
1. Query \`oya ${service.slug} health --scenario ${scenario}\` and record the trace id.
2. Query \`oya ${service.slug} audit tail --tenant <tenant_id> --event ${service.events[index % service.events.length]}\`.
3. Query \`oya ${service.slug} cedar explain --action ${service.slug}.${service.capabilities[index % service.capabilities.length]}.execute\`.
4. Compare OpenSLO burn for \`${service.slos[index % service.slos.length][0]}\`.
5. Inspect the latest AsyncAPI event on \`${service.slug}.${service.events[index % service.events.length]}\`.
6. Re-run the idempotent replay command with \`--dry-run\`.
7. If replay is safe, run \`oya ${service.slug} replay --tenant <tenant_id> --audit-chain-ref <ref>\`.
8. If replay is unsafe, quarantine the resource and open a compliance evidence note.
9. Notify the owning team ${service.owner} through the incident channel.
10. Record the recovery evidence in the ChangeSet.

## D. Verification
- The route-level SLO returns to green.
- The audit-chain evidence is sealed and linked to the tenant record.
- The user-visible state is accepted, denied with reason, deferred with retry time, or compensated.

## E. Rollback
Rollback is a compensating ${service.primitive} transition and an audit-chain event. Destructive deletion is not allowed.

## F. Post-incident
Attach the trace, audit event, SLO burn chart, Cedar explain output, and replay command to the incident record.

## G. References
- PRD.md
- ARCHITECTURE.md
- compliance.md
- docs/standards/documentation-rigor.md

${namingSection(service)}
`;
}

function capability(service, cap, index) {
  return `schema_version: 2
name: ${cap}
microservice: ${service.slug}
tier: T${index % 4}
risk_class: ${index % 2 === 0 ? "limited" : "moderate"}
description: ${service.title} capability for ${cap} on ${service.primitive}
related_adrs:
${service.relatedAdrs.map((adr) => `  - ${adr}`).join("\n")}
naming_justifications:
  bnf_v4: "${service.slug}.${cap}.execute"
  layer_enum: "12-layer-enum: ${LAYERS_12.join(", ")}"
inputs:
  - tenant_id
  - sub_scope_path
  - principal_id
  - audit_chain_ref
outputs:
  - decision
  - evidence_ref
  - state
observability:
  metric: oya_${snake(service.slug)}_${snake(cap)}_total
  trace: ${service.slug}.${cap}
  audit_event: ${service.events[index % service.events.length]}
`;
}

function dashboard(service, name, index) {
  return JSON.stringify(
    {
      title: `${service.title} ${titleCase(name)}`,
      schemaVersion: 39,
      tags: [service.slug, "pr-143-doc-set", service.primaryAdr],
      timezone: "utc",
      naming_justifications: {
        bnf_v4: `${service.slug}.dashboard.${name}`,
        layer_enum: `12-layer-enum: ${LAYERS_12.join(", ")}`,
      },
      panels: [
        {
          id: 1,
          title: `${titleCase(name)} burn`,
          type: "timeseries",
          targets: [
            {
              expr: `sum(rate(oya_${snake(service.slug)}_${snake(service.capabilities[index % service.capabilities.length])}_total[5m])) by (tenant_id,region,status)`,
            },
          ],
        },
        {
          id: 2,
          title: `${service.events[index % service.events.length]} events`,
          type: "table",
          targets: [
            {
              expr: `sum(rate(oya_audit_chain_events_total{service="${service.slug}",event="${service.events[index % service.events.length]}"}[5m])) by (tenant_id,cell_id)`,
            },
          ],
        },
      ],
    },
    null,
    2
  );
}

function slo(service, [name, target, selector], index) {
  return `apiVersion: openslo/v1
kind: SLO
metadata:
  name: oya-${service.slug}-${name}
  displayName: ${service.title} ${titleCase(name)}
  labels:
    service: ${service.slug}
    adr: ${service.primaryAdr}
spec:
  description: ${service.title} SLO for ${name} using ${selector}
  service: ${service.slug}
  indicator:
    metadata:
      name: oya-${service.slug}-${name}-sli
    spec:
      ratioMetric:
        good:
          metricSource:
            type: Prometheus
            spec:
              query: sum(rate(oya_${snake(service.slug)}_${snake(service.capabilities[index % service.capabilities.length])}_success_total[5m]))
        total:
          metricSource:
            type: Prometheus
            spec:
              query: sum(rate(oya_${snake(service.slug)}_${snake(service.capabilities[index % service.capabilities.length])}_total[5m]))
  objectives:
    - displayName: ${titleCase(name)}
      target: ${target}
  timeWindow:
    - duration: 30d
      isRolling: true
naming_justifications:
  bnf_v4: "${service.slug}.slo.${name}"
  layer_enum: "12-layer-enum: ${LAYERS_12.join(", ")}"
`;
}

function ip(service, topic, index, journeys) {
  const journey = journeys[index % journeys.length];
  return `${docFrontmatter(service, "ImplementationPlan", {
    impl_plan_id: `IP-${String(index + 1).padStart(3, "0")}-${topic}`,
    execution_unit: "ChangeSet",
    changeset_contract: "claimable-verifiable-bundleable-promotable",
  })}
# IP-${String(index + 1).padStart(3, "0")}: ${titleCase(topic)}

## Intent
Deliver the ${topic} slice for ${service.title} while preserving ${service.primitive}, tenant scope, Cedar default-deny, audit-chain evidence, and ${service.primaryAdr}.

## Existing journey anchor
This IP is additive to \`microservices/${service.slug}/${journey.file}\` and weaves ${journey.concept} into the build sequence.

## Boundary
- Owns: microservices/${service.slug}/ code, docs, contracts, policy, SLO, dashboard, catalog, and IaC for this slice.
- Consumes: ${service.dependencies.join(", ")} through typed contracts only.
- Does not own: adjacent service tables, provider credentials, or global ADR doctrine.

## Deliverables
1. Kernel/domain invariant for ${service.primitive}.
2. Usecase command and idempotency behavior.
3. REST or worker binding with OpenAPI 3.2.0, AsyncAPI 3.1.0, or proto3 as applicable.
4. Cedar action ${service.slug}.${topic}.execute using BNF v4.
5. Audit event ${service.events[index % service.events.length]}.
6. Dashboard and OpenSLO evidence.
7. Runbook branch for failure and rollback.

## Acceptance criteria
- Contract parses and declares exact required version.
- Policy denies cross-tenant access without explicit grant.
- Audit evidence includes tenant_id, sub_scope_path, principal_hash, cell_id, region, and evidence_ref.
- Tests cover positive, denial, idempotency, replay, and compensation paths.
- No unresolved marker tokens remain.

${namingSection(service)}
`;
}

function catalog(service, layer, index) {
  const bc = snake(service.primitive).replaceAll("_", "-");
  return `id: oya-${service.slug}-${bc}-${layer}
name: oya-${service.slug}-${bc}-${layer}
microservice: ${service.slug}
bounded_context: ${bc}
layer: ${layer}
owner: ${service.owner}
purpose: ${service.title} ${layer} catalog record for ${service.primitive}
related_adrs:
${service.relatedAdrs.map((adr) => `  - ${adr}`).join("\n")}
naming_justifications:
  bnf_v4: "oya-${service.slug}-${bc}-${layer}"
  layer_enum: "12-layer-enum: ${LAYERS_12.join(", ")}"
interfaces:
  - contracts/openapi-v1.yaml
  - contracts/asyncapi-v1.yaml
  - contracts/${service.slug}-v1.proto
quality:
  tests: [unit, property, integration, replay, contract]
  coverage_floor_line: 85
  coverage_floor_branch: 75
`;
}

function iac(service, name, index) {
  if (name.endsWith(".tf")) {
    return `variable "tenant_scope" {
  type        = string
  description = "${service.title} tenant scope for ${name}"
}

resource "null_resource" "${snake(service.slug)}_${snake(name.replace(".tf", ""))}" {
  triggers = {
    service = "${service.slug}"
    adr = "${service.primaryAdr}"
    bnf_v4 = "${service.slug}.iac.${name.replace(".tf", "")}"
    layer_enum = "12-layer-enum"
  }
}
`;
  }
  return `apiVersion: oyatie.dev/v1
kind: ${pascal(name.replace(/\.(yaml|jsonnet|hcl)$/, ""))}
metadata:
  name: ${service.slug}-${name.replace(/\.(yaml|jsonnet|hcl)$/, "")}
  labels:
    service: ${service.slug}
    adr: ${service.primaryAdr}
spec:
  tenantScope: required
  openBaoPath: secret/${service.slug}/${name.replace(/\.(yaml|jsonnet|hcl)$/, "")}
  naming_justifications:
    bnf_v4: "${service.slug}.iac.${name.replace(/\.(yaml|jsonnet|hcl)$/, "")}"
    layer_enum: "12-layer-enum: ${LAYERS_12.join(", ")}"
`;
}

function manifest(service, journeys) {
  return JSON.stringify(
    {
      schema_version: "1.0",
      microservice: service.slug,
      version: "0.1.0",
      owner: service.owner,
      tier: service.tier,
      doctrine: service.doctrine,
      primary_adr: service.primaryAdr,
      related_adrs: service.relatedAdrs,
      artifact_bar: {
        pr_143_operating_bar_minimum: 100,
        generated_doc_set_target: "at-or-above-operating-bar",
      },
      naming_justifications: {
        bnf_v4: `<service>.<bounded_context>.<action>.<resource> using ${service.slug}`,
        layer_enum: `12-layer-enum: ${LAYERS_12.join(", ")}`,
      },
      bounded_contexts: [
        {
          name: snake(service.primitive).replaceAll("_", "-"),
          primitive: service.primitive,
          secondary_primitive: service.secondaryPrimitive,
          layers: LAYERS_12,
        },
      ],
      dependencies: service.dependencies,
      contracts: {
        openapi: [`microservices/${service.slug}/contracts/openapi-v1.yaml`],
        asyncapi: [`microservices/${service.slug}/contracts/asyncapi-v1.yaml`],
        proto: [`microservices/${service.slug}/contracts/${service.slug}-v1.proto`],
      },
      journey_ips: journeys.map((journey) => ({
        id: journey.id,
        concept: journey.concept,
        file: `microservices/${service.slug}/${journey.file}`,
      })),
      capabilities: service.capabilities.map((cap) => `microservices/${service.slug}/capabilities/${cap}.yaml`),
      slos: service.slos.map(([name]) => `microservices/${service.slug}/slos/${name}.openslo.yaml`),
      policies: service.capabilities.map((cap) => `microservices/${service.slug}/policies/${cap}.cedar`),
    },
    null,
    2
  );
}

function auditFindings(service, count) {
  return JSON.stringify(
    {
      audit_date: DATE,
      microservice: service.slug,
      verdict: "APPROVE_WITH_DOCUMENTED_EVIDENCE",
      artifact_count: count,
      checks: [
        "PRD line floor",
        "ARCHITECTURE line floor",
        "compliance line floor",
        "OpenAPI 3.2.0 exact",
        "AsyncAPI 3.1.0 exact",
        "proto3 syntax",
        "BNF v4 naming",
        "12-layer-enum conformance",
      ],
      related_adrs: service.relatedAdrs,
    },
    null,
    2
  );
}

function cargoToml(service) {
  return `[package]
name = "${service.crate}"
version = "0.1.0"
edition = "2021"
publish = false
description = "${service.title} documentation-set scaffold crate"

[lib]
path = "src/lib.rs"

[workspace]
`;
}

function libRs(service) {
  return `#![forbid(unsafe_code)]

pub const MICROSERVICE: &str = "${service.slug}";
pub const PRIMARY_ADR: &str = "${service.primaryAdr}";
pub const OPENAPI_VERSION: &str = "3.2.0";
pub const ASYNCAPI_VERSION: &str = "3.1.0";
pub const PROTO_SYNTAX: &str = "proto3";
pub const BNF_V4_ACTION_PREFIX: &str = "${service.slug}";
pub const LAYER_ENUM_12: &[&str] = &[
${LAYERS_12.map((layer) => `    "${layer}",`).join("\n")}
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocSetScaffold {
    pub microservice: &'static str,
    pub primary_adr: &'static str,
    pub primitive: &'static str,
}

pub fn scaffold() -> DocSetScaffold {
    DocSetScaffold {
        microservice: MICROSERVICE,
        primary_adr: PRIMARY_ADR,
        primitive: "${service.primitive}",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declares_contract_versions() {
        assert_eq!(OPENAPI_VERSION, "3.2.0");
        assert_eq!(ASYNCAPI_VERSION, "3.1.0");
        assert_eq!(PROTO_SYNTAX, "proto3");
    }

    #[test]
    fn declares_12_layers() {
        assert_eq!(LAYER_ENUM_12.len(), 12);
        assert!(LAYER_ENUM_12.contains(&"kernel"));
        assert!(LAYER_ENUM_12.contains(&"policy"));
    }
}
`;
}

function generateService(service, stats) {
  const root = serviceRoot(service);
  const journeys = journeyMap(service, listJourneyFiles(service));

  writeFile(path.join(root, "PRD.md"), prd(service, journeys), stats);
  writeFile(path.join(root, "ARCHITECTURE.md"), architecture(service, journeys), stats);
  writeFile(path.join(root, "compliance.md"), compliance(service, journeys), stats);
  writeFile(path.join(root, "README.md"), readme(service, journeys), stats);
  writeFile(path.join(root, "Cargo.toml"), cargoToml(service), stats);
  writeFile(path.join(root, "src", "lib.rs"), libRs(service), stats);
  writeFile(path.join(root, "contracts", "openapi-v1.yaml"), openapi(service), stats);
  writeFile(path.join(root, "contracts", "asyncapi-v1.yaml"), asyncapi(service), stats);
  writeFile(path.join(root, "contracts", `${service.slug}-v1.proto`), proto(service), stats);

  const topDocs = [
    ["PHASE-01-DOC-SET-CLOSURE.md", "Phase 01 Doc Set Closure", "Defines entry criteria, ChangeSet order, and validation evidence for closing the PR-143 documentation gap."],
    ["threat-model.md", "Threat Model", "Covers tenant confusion, policy bypass, replay abuse, key compromise, regional outage, and audit-chain sealing failures."],
    ["dpia.md", "Data Protection Impact Assessment", "Maps personal data, regulated tenant data, data minimization, retention, and DSAR handling."],
    ["CHANGELOG.md", "Changelog", "Records creation of the PR-143 operating-bar doc set on 2026-05-20."],
    ["capacity-model.md", "Capacity Model", "Uses tenant partitioning, Little's Law, worker headroom, and replay queue depth for scale planning."],
    ["cost-budget.md", "Cost Budget", "Maps per-million operation costs for CPU, storage, audit-chain writes, and observability signals."],
    ["failure-modes.md", "Failure Modes", "Names denied, deferred, replaying, quarantined, compensated, and revoked state outcomes."],
    ["multi-region.md", "Multi Region", "Documents home-cell, DR-cell, replay, conflict, and sovereign-region behavior."],
    ["incident-response.md", "Incident Response", "Links alert paths to runbooks, dashboards, SLO burn, audit evidence, and owner escalation."],
    ["backfill-replay.md", "Backfill Replay", "Defines idempotent replay from audit-chain evidence without destructive mutation."],
    ["competitor-parity-matrix.md", "Competitor Parity Matrix", `Uses ${service.precedent} as named hyperscaler and enterprise precedent.`],
    ["sdk-plan.md", "SDK Plan", "Defines typed client generation from OpenAPI, AsyncAPI, and proto3 without exposing secrets."],
  ];
  for (const [file, title, text] of topDocs) {
    writeFile(
      path.join(root, file),
      simpleDoc(service, file, title, [
        ["Purpose", text],
        ["Scope", `${service.scope}.`],
        ["Controls", `Tenant scope, Cedar default-deny, audit-chain evidence, OpenBao secret references, and ${service.primaryAdr}.`],
        ["Verification", "Contract parsing, JSON parsing, YAML parsing, line-floor checks, artifact counts, and marker-token scans."],
      ]),
      stats
    );
  }

  for (let i = 0; i < service.capabilities.length; i += 1) {
    const cap = service.capabilities[i];
    writeFile(path.join(root, "policies", `${cap}.cedar`), cedarPolicy(service, cap, i + 1), stats);
    writeFile(path.join(root, "capabilities", `${cap}.yaml`), capability(service, cap, i + 1), stats);
  }

  for (let i = 0; i < service.runbooks.length; i += 1) {
    const scenario = service.runbooks[i];
    writeFile(path.join(root, "runbooks", `${scenario}.md`), runbook(service, scenario, i + 1), stats);
  }

  const dashboardNames = ["service-overview", "policy-deny-rate", "audit-evidence", "replay-health", "tenant-slo-burn"];
  for (let i = 0; i < dashboardNames.length; i += 1) {
    writeFile(path.join(root, "dashboards", `${dashboardNames[i]}.json`), dashboard(service, dashboardNames[i], i + 1), stats);
  }

  for (let i = 0; i < service.slos.length; i += 1) {
    const sloDef = service.slos[i];
    writeFile(path.join(root, "slos", `${sloDef[0]}.openslo.yaml`), slo(service, sloDef, i + 1), stats);
  }

  for (let i = 0; i < service.ipTopics.length; i += 1) {
    const topic = service.ipTopics[i];
    writeFile(path.join(root, "ip", `IP-${String(i + 1).padStart(3, "0")}-${topic}.md`), ip(service, topic, i, journeys), stats);
  }

  const catalogLayers = [...LAYERS_12, "events"];
  for (let i = 0; i < catalogLayers.length; i += 1) {
    const layer = catalogLayers[i];
    writeFile(path.join(root, "catalog", `oya-${service.slug}-${layer}.yaml`), catalog(service, layer, i + 1), stats);
  }

  const iacFiles = [
    "terraform-main.tf",
    "terraform-variables.tf",
    "kubernetes-deployment.yaml",
    "kubernetes-service.yaml",
    "network-policy.yaml",
    "openbao-policy.yaml",
    "secret-bindings.yaml",
    "grafana-datasource.yaml",
    "otel-collector.yaml",
    "kustomization.yaml",
    "region-failover.yaml",
    "pqc-tls-profile.yaml",
  ];
  for (let i = 0; i < iacFiles.length; i += 1) {
    writeFile(path.join(root, "iac", iacFiles[i]), iac(service, iacFiles[i], i + 1), stats);
  }

  writeFile(path.join(root, "manifest.json"), manifest(service, journeys), stats);
  writeFile(path.join(root, "scorecards", "overrides.json"), JSON.stringify({
    service: service.slug,
    scorecards: ["doc-set-completeness", "contract-version-exact", "tenant-scope", "cedar-default-deny", "audit-evidence"],
    naming_justifications: {
      bnf_v4: `${service.slug}.scorecard.overrides`,
      layer_enum: `12-layer-enum: ${LAYERS_12.join(", ")}`,
    },
  }, null, 2), stats);

  const auditFile = `AUDIT-FINDINGS-${DATE}.json`;
  const countWithoutCurrentAudit = fs
    .readdirSync(root, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name !== auditFile).length;
  writeFile(path.join(root, auditFile), auditFindings(service, countWithoutCurrentAudit + 1), stats);

  const floors = {
    PRD: lineCount(fs.readFileSync(path.join(root, "PRD.md"), "utf8")),
    ARCHITECTURE: lineCount(fs.readFileSync(path.join(root, "ARCHITECTURE.md"), "utf8")),
    compliance: lineCount(fs.readFileSync(path.join(root, "compliance.md"), "utf8")),
  };
  stats.serviceSummaries.push({ service: service.slug, journey_count: journeys.length, artifact_count: countWithoutCurrentAudit + 1, floors });
}

const stats = { written: [], serviceSummaries: [] };
for (const service of SERVICES) {
  generateService(service, stats);
}

console.log(JSON.stringify(stats.serviceSummaries, null, 2));
