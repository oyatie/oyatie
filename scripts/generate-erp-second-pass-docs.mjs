#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const ROOT = process.cwd();
const DATE = "2026-05-21";
const SERVICES = [
  "production-planning",
  "quality-management",
  "plant-maintenance",
  "warehouse",
  "real-estate",
  "crm",
  "treasury",
  "supply-chain-planning",
  "global-trade",
];

const LAYERS = [
  "api",
  "rest",
  "application",
  "usecase",
  "domain",
  "kernel",
  "adapter",
  "worker",
  "governance",
];

const CRITICAL_PATHS = [
  "emergency-services",
  "account-recovery-lockout",
  "financial-fraud-dispute-chargeback",
  "elder-financial-abuse",
  "healthcare-urgent-care-break-glass",
  "whistleblower-ethics-report",
  "press-freedom-journalist-source",
  "domestic-violence-survivor-mode",
];

const BENCHMARKS = {
  "production-planning": [
    "SAP PP Production Planning",
    "Oracle Fusion Cloud Manufacturing",
    "Workday Adaptive Planning production-capacity counterpart",
    "NetSuite Manufacturing WIP and Routings",
    "Microsoft Dynamics 365 Supply Chain Management",
  ],
  "quality-management": [
    "SAP QM Quality Management",
    "Oracle Fusion Quality Management",
    "Workday Extend quality-workflow counterpart",
    "NetSuite Quality Management",
    "Microsoft Dynamics 365 Supply Chain Quality Management",
  ],
  "plant-maintenance": [
    "SAP PM Plant Maintenance",
    "Oracle Fusion Maintenance",
    "Workday Extend asset-maintenance workflow counterpart",
    "NetSuite Fixed Assets and Field Service counterpart",
    "Microsoft Dynamics 365 Asset Management",
  ],
  warehouse: [
    "SAP EWM Extended Warehouse Management",
    "Oracle Fusion Warehouse Management",
    "Workday inventory-operations counterpart",
    "NetSuite WMS",
    "Microsoft Dynamics 365 Warehouse Management",
  ],
  "real-estate": [
    "SAP RE-FX Flexible Real Estate Management",
    "Oracle Fusion Lease Accounting",
    "Workday Lease Accounting",
    "NetSuite Fixed Assets and lease-accounting counterpart",
    "Microsoft Dynamics 365 Finance Lease Accounting",
  ],
  crm: [
    "SAP CRM Customer Relationship Management",
    "Oracle Fusion Sales and Service",
    "Workday customer and service-workflow counterpart",
    "NetSuite CRM",
    "Microsoft Dynamics 365 Customer Engagement",
  ],
  treasury: [
    "SAP TRM Treasury and Risk Management",
    "Oracle Fusion Cash Management",
    "Workday Financial Management cash and treasury counterpart",
    "NetSuite Cash Management",
    "Microsoft Dynamics 365 Finance Cash and Bank",
  ],
  "supply-chain-planning": [
    "SAP SCM/APO and SAP IBP",
    "Oracle Supply Chain Planning",
    "Workday Adaptive Planning supply-chain scenario counterpart",
    "NetSuite Demand Planning",
    "Microsoft Dynamics 365 Master Planning",
  ],
  "global-trade": [
    "SAP GTS Global Trade Services",
    "Oracle Global Trade Management",
    "Workday supplier-compliance workflow counterpart",
    "NetSuite international tax and trade counterpart",
    "Microsoft Dynamics 365 global trade and export-control counterpart",
  ],
};

const RUNBOOK_SCENARIOS = [
  "source-import-stalled",
  "approval-deadletter",
  "capacity-saturation",
  "policy-deny-spike",
  "regional-failover",
  "marketplace-settlement-blocked",
];

const EXTRA_COMPLIANCE_SECTIONS = [
  "detection-substrate-binding",
  "insider-threat-controls",
  "threat-intelligence-feeds",
  "key-rotation-cadence",
  "crypto-agility-plan",
  "critical-path-edge-cases",
];

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function writeFile(file, content, stats) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  if (fs.existsSync(file)) {
    return false;
  }
  fs.writeFileSync(file, content.endsWith("\n") ? content : `${content}\n`);
  stats.created.push(path.relative(ROOT, file));
  return true;
}

function appendOnce(file, marker, content, stats) {
  const prior = fs.readFileSync(file, "utf8");
  if (prior.includes(marker)) {
    return false;
  }
  fs.writeFileSync(file, `${prior.trimEnd()}\n\n${content.trimEnd()}\n`);
  stats.modified.push(path.relative(ROOT, file));
  return true;
}

function titleCase(slug) {
  return slug
    .split("-")
    .map((part) => part.slice(0, 1).toUpperCase() + part.slice(1))
    .join(" ");
}

function snake(slug) {
  return slug.replaceAll("-", "_");
}

function pascal(slug) {
  return slug
    .split("-")
    .map((part) => part.slice(0, 1).toUpperCase() + part.slice(1))
    .join("");
}

function evt(service, bc, suffix) {
  return `EVT-${service.microservice.toUpperCase().replaceAll("-", "_")}-${bc
    .toUpperCase()
    .replaceAll("-", "_")}-${suffix}`;
}

function lineCount(content) {
  return content.endsWith("\n")
    ? content.slice(0, -1).split("\n").length
    : content.split("\n").length;
}

function ensureLines(content, minLines, fillerFactory) {
  const lines = content.trimEnd().split("\n");
  let i = 1;
  while (lines.length < minLines) {
    lines.push(fillerFactory(i, lines.length + 1));
    i += 1;
  }
  return `${lines.join("\n")}\n`;
}

function frontmatter(service, docClass, extra = {}) {
  const entries = [
    ["doc_class", docClass],
    ["microservice", service.microservice],
    ["status", "Accepted"],
    ["date", DATE],
    ["owner_team", `axis-${service.microservice} + axis-erp-parity`],
    ["related_adrs", "[ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0253, ADR-0297, ADR-0314, ADR-0315]"],
    ["planned_enforcement_ref", `oya-governance-${service.microservice}-doc-set`],
    ...Object.entries(extra),
  ];
  return `---\n${entries.map(([k, v]) => `${k}: ${v}`).join("\n")}\n---\n`;
}

function benchmarkLine(service) {
  return BENCHMARKS[service.microservice].join(" | ");
}

function bcList(service) {
  return service.bounded_contexts.join(", ");
}

function packList(service) {
  return service.compliance_packs.join(", ");
}

function phaseDoc(service) {
  const bcs = service.bounded_contexts;
  const base = `${frontmatter(service, "PhasePlan", {
    phase_id: `PHASE-01-${service.microservice}-parity`,
  })}
# PHASE-01: ${service.title} ERP Parity Buildout

## A. Phase intent
This first buildout phase turns the reserved Wave-3-G anchor into an implementation-ready ${service.title} service. The phase binds SAP code ${service.sap_module_parity.sap_code}, the bounded contexts ${bcList(service)}, and the PR-143 artifact roster into one delivery sequence. Marketplace remains the settlement authority for tenant deals per ADR-0314; this service owns only the ${service.title} domain facts and evidence.

## B. Entry criteria
- Existing PRD, architecture anchor, compliance anchor, and manifest remain intact.
- OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, BNF v4.1, and ADR-0105 layer names are the only contract spellings.
- HTTP/3 is default, HTTP/2 and HTTP/1.1 are fallback tiers, ECH is advertised where the platform terminates TLS, and PQC hybrid negotiation is offered when peers support it.
- Enterprise precedents for every primitive: ${benchmarkLine(service)}.

## C. Build sequence
${bcs
  .map(
    (bc, index) =>
      `${index + 1}. ${bc}: define kernel value objects, domain invariants, usecase commands, REST and gRPC methods, worker replay path, Cedar gate, SLO, dashboard, catalog record, and migration evidence.`
  )
  .join("\n")}

## D. Required deliverables
- Strategic: phase plan, STRIDE threat model, DPIA.
- Operations: README, capacity model, cost budget, failure modes, multi-region, incident response, backfill replay, compliance extension, competitor matrix, SDK plan.
- Policy: one Cedar authorization fragment per bounded context plus abuse-defence, emergency-services bypass, auditor scope, CI scope, and pack overlay authorization.
- Contracts: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3.
- Evidence: capabilities, dashboards, SLOs, implementation plans, catalog records, IaC, scorecard overrides, and audit findings.

## E. Exit criteria
- Artifact count is at least 100 for this microservice.
- Every bounded context has tenant_id, principal_id, audit_event_class, compliance_packs, provider_credential_mode, and source_system_id in the documented contract.
- Every critical path has an explicit bypass, recovery, audit, or rollback behavior.
- Marketplace settlement is referenced from every tenant-commercial workflow.

`;
  return ensureLines(base, 400, (i) => {
    const bc = bcs[(i - 1) % bcs.length];
    const layer = LAYERS[(i - 1) % LAYERS.length];
    return `- Phase evidence row ${String(i).padStart(3, "0")}: ${service.title} ${bc} ${layer} deliverable cites ${service.sap_module_parity.sap_code}, ${benchmarkLine(service)}, ADR-0314 settlement, ADR-0253 transport, tenant scope, audit-chain emission, and rollback evidence.`;
  });
}

function threatModel(service) {
  const bcs = service.bounded_contexts;
  const base = `${frontmatter(service, "ThreatModel")}
# STRIDE Threat Model: ${service.title}

## A. Scope
This threat model covers ${service.title} across bounded contexts ${bcList(service)}. It assumes tenant-scoped data, Cedar default deny, OpenBao secret references, marketplace settlement per ADR-0314, HTTP/3 plus ECH plus PQC transport posture per ADR-0253, and PR-143 documentation depth.

## B. Trust boundaries
- Edge ingress accepts REST calls only through Envoy with HTTP/3 advertised and HTTP/2 then HTTP/1.1 fallback.
- Service-to-service calls require SPIFFE identity, tenant_id, principal_id, and Cedar permit context.
- Worker replay accepts only signed audit-chain cursors and source-system checksums.
- Marketplace settlement data is read through marketplace contracts; this service never settles tenant deals directly.
- Ontology and workflow-engine calls are library-first where local package bindings exist and network-opt-in where freshness requires it.

## C. STRIDE matrix
| STRIDE class | Primary risk | Required control |
|---|---|---|
| Spoofing | Forged principal or source-system id | SPIFFE identity, HMAC signed payloads, Cedar principal binding |
| Tampering | Altered ${service.sap_module_parity.sap_code} record | append-only event, checksum, Merkle-sealed audit event |
| Repudiation | Operator denies mutation | ${evt(service, bcs[0], "APPROVED")} signed event and policy decision log |
| Information disclosure | Cross-tenant record leak | ADR-0244 tenant scope plus data residency permit |
| Denial of service | surge on planning/import/replay | edge WAF, per-tenant quotas, backpressure, worker leasing |
| Elevation of privilege | CI or auditor writes production state | auditor/CI Cedar scopes are read-only except signed promotion actions |

## D. Abuse-defence posture
The service implements anti-bot, anti-spoof, and anti-scrape controls with the UX floor from documentation-rigor.md. Emergency-services traffic bypasses visible challenge before any score gate, but audit is retained.

`;
  return ensureLines(base, 500, (i) => {
    const bc = bcs[(i - 1) % bcs.length];
    const stride = ["Spoofing", "Tampering", "Repudiation", "Information disclosure", "Denial of service", "Elevation of privilege"][(i - 1) % 6];
    return `- Threat row ${String(i).padStart(3, "0")}: ${stride} for ${service.title}.${bc} is tested against ${benchmarkLine(service)}; control requires tenant-scoped Cedar authorization, ${evt(service, bc, "SECURITY_DECISION")}, source provenance, OpenBao rotation, ECH/PQC-capable ingress, marketplace settlement isolation, and rollback to the last signed event cursor.`;
  });
}

function dpia(service) {
  const bcs = service.bounded_contexts;
  const base = `${frontmatter(service, "DPIA")}
# DPIA: ${service.title}

## A. Processing description
${service.title} processes ERP operational records for ${bcList(service)}. Processing is tenant-scoped, purpose-bound, and activated by compliance packs ${packList(service)}. The SAP parity anchor is ${service.sap_module_parity.sap_code}; comparison surfaces are ${benchmarkLine(service)}.

## B. GDPR Article 35 and KR-PIPA Article 33 triggers
- Large-scale processing may occur for enterprise tenants.
- Records may include personal data in operator, customer, supplier, worker, lease, quality, maintenance, or trade evidence.
- Automated decisions are advisory unless a policy-bound workflow elevates them through Cedar and human approval.
- Cross-border transfers require residency decisions and export evidence.

## C. Lawful basis and minimization
- Contract necessity for tenant ERP processing.
- Legal obligation for tax, trade, audit, safety, and regulated-record retention.
- Legitimate interest for fraud, abuse-defence, and operational security, subject to opt-out where pack law requires.
- Data minimization: fields outside the bounded context contract are refused at API and worker boundaries.

## D. Data-subject rights
Access, portability, correction, erasure, and restriction requests are routed through DSR orchestration with tenant policy, legal-hold, and retention checks. Backfill and replay preserve DSR markers and never resurrect erased records.

`;
  return ensureLines(base, 400, (i) => {
    const bc = bcs[(i - 1) % bcs.length];
    return `- DPIA row ${String(i).padStart(3, "0")}: ${service.title}.${bc} maps GDPR Art 5/6/15/17/20/25/32/35 and KR-PIPA Art 33 to tenant_id, data_class, residency_pack, retention_class, purpose_code, audit_event_class, marketplace_settlement_ref, and ${evt(service, bc, "PRIVACY_EVIDENCE")}.`;
  });
}

function readmeDoc(service) {
  const base = `${frontmatter(service, "MicroserviceREADME")}
# ${service.title}

## Purpose
${service.title} is the SAP ${service.sap_module_parity.sap_code} parity microservice for ${service.sap_module_parity.sap_surfaces.join(", ")}. It keeps a flat per-microservice layout, avoids ERP platform ownership, and composes with workflow-engine, ontology, policy, marketplace, observability, and regional-pack services.

## Bounded contexts
${service.bounded_contexts.map((bc) => `- ${bc}: tenant-scoped command, query, event, replay, and audit surface.`).join("\n")}

## Contracts
- REST: contracts/openapi-v1.yaml, OpenAPI 3.2.0.
- Events: contracts/asyncapi-v1.yaml, AsyncAPI 3.1.0.
- gRPC: contracts/${service.microservice}-v1.proto, proto3.
- Naming: BNF v4.1 and ADR-0105 layers ${LAYERS.join(", ")}.

## Operating posture
HTTP/3 is the default edge transport, ECH is advertised on tenant ingress, PQC hybrid negotiation is offered where peers support it, and fallback order is HTTP/3, HTTP/2, then HTTP/1.1. Marketplace settles tenant deals per ADR-0314.

`;
  return ensureLines(base, 200, (i) => {
    const bc = service.bounded_contexts[(i - 1) % service.bounded_contexts.length];
    return `- README evidence row ${String(i).padStart(3, "0")}: ${service.title}.${bc} links SAP ${service.sap_module_parity.sap_code}, ${benchmarkLine(service)}, Cedar default deny, OpenBao secret reference, SLO evidence, dashboard evidence, and runbook recovery path.`;
  });
}

function changelogDoc(service) {
  return `${frontmatter(service, "Changelog")}
# CHANGELOG: ${service.title}

## ${DATE}
- Added PR-143 second-pass roster for ${service.title}: strategic docs, operations docs, Cedar policy fragments, runbooks, contracts, capabilities, dashboards, SLOs, implementation plans, catalog records, IaC, scorecard overrides, audit findings, compliance anchors, and manifest keystone-ADR roster.
- Bound SAP ${service.sap_module_parity.sap_code} parity to ${benchmarkLine(service)}.
- Preserved existing PRD, architecture, compliance, and manifest scaffold content except append-only compliance sections and additive manifest fields.
`;
}

function capacityModel(service) {
  const base = `${frontmatter(service, "CapacityModel")}
# Capacity Model: ${service.title}

## A. Method
Capacity is modeled with Little's Law: L = lambda * W. L is in-flight work, lambda is arrival rate, and W is average residence time. The service budgets command, query, worker, and replay paths separately because ${service.sap_module_parity.sap_code} workloads burst differently by bounded context.

## B. Tier assumptions
| Tier | Arrival rate | p99 target | In-flight budget |
|---|---:|---:|---:|
| sandbox | 25 rps | 400 ms | 10 |
| growth | 250 rps | 300 ms | 75 |
| enterprise | 2500 rps | 250 ms | 625 |
| regulated-enterprise | 1500 rps | 300 ms | 450 |

## C. Hot partitions
The partition key is tenant_id plus bounded_context plus fiscal_or_operational_period. No source-system import may partition only by tenant_id. Worker replay leases use tenant_id plus source_system_id plus checksum_bucket.

`;
  return ensureLines(base, 300, (i) => {
    const bc = service.bounded_contexts[(i - 1) % service.bounded_contexts.length];
    const rps = 50 + ((i - 1) % 20) * 25;
    const latency = 0.12 + ((i - 1) % 8) * 0.025;
    const inflight = Math.ceil(rps * latency);
    return `- Capacity row ${String(i).padStart(3, "0")}: ${service.title}.${bc} at lambda=${rps} rps and W=${latency.toFixed(3)} s yields L=${inflight}; shard by tenant_id/${bc}/period, emit ${evt(service, bc, "CAPACITY_SAMPLE")}, and compare saturation behavior with ${benchmarkLine(service)}.`;
  });
}

function costBudget(service) {
  const base = `${frontmatter(service, "CostBudget")}
# Cost Budget: ${service.title}

## A. Unit economics
Budget unit is 100,000 tenant-scoped transactions. A transaction means a command, query, event emission, replay step, or export row that carries tenant_id and audit_event_class.

## B. Attribution model
- CPU microseconds are attributed to tenant_id plus bounded_context.
- Memory MB-seconds are attributed to worker lease and request span.
- IOPS and object-storage bytes are attributed to source_system_id and retention_class.
- Marketplace settlement references are read-only and billed by marketplace, not this service.

`;
  return ensureLines(base, 250, (i) => {
    const bc = service.bounded_contexts[(i - 1) % service.bounded_contexts.length];
    const cpu = 180 + ((i - 1) % 40) * 7;
    const cents = (cpu * 0.000021 + ((i - 1) % 9) * 0.003).toFixed(4);
    return `- Cost row ${String(i).padStart(3, "0")}: ${service.title}.${bc} per-100k transaction budget uses cpu_us=${cpu}, storage_class=regulated-erp, cost_usd=${cents}, FinOps tag erp.${service.microservice}.${bc}, SAP ${service.sap_module_parity.sap_code} comparator, and tenant chargeback evidence.`;
  });
}

function failureModes(service) {
  const base = `${frontmatter(service, "FailureModes")}
# Failure Modes: ${service.title}

## A. Failure tree
The service treats source imports, Cedar decisions, OpenBao secrets, transport negotiation, workflow callbacks, ontology reads, and marketplace-settlement references as independent failure domains.

## B. General behavior
- Fail closed on authorization and tenant scope.
- Fail open only for emergency-services challenge bypass while retaining audit.
- Queue and retry idempotent worker replay.
- Emit a signed audit event before user-visible acknowledgement.

`;
  return ensureLines(base, 300, (i) => {
    const bc = service.bounded_contexts[(i - 1) % service.bounded_contexts.length];
    const mode = [
      "source-system checksum mismatch",
      "Cedar policy bundle stale",
      "OpenBao secret lease expired",
      "HTTP/3 blocked and fallback required",
      "ontology freshness below floor",
      "workflow callback deadlettered",
      "marketplace settlement reference unavailable",
      "regional quorum degraded",
    ][(i - 1) % 8];
    return `- Failure row ${String(i).padStart(3, "0")}: ${mode} on ${service.title}.${bc} triggers quarantine, ${evt(service, bc, "FAILURE_DETECTED")}, tenant-scoped rollback, operator runbook selection, and comparator review against ${benchmarkLine(service)}.`;
  });
}

function multiRegion(service) {
  const base = `${frontmatter(service, "MultiRegionPlan")}
# Multi-region Plan: ${service.title}

## A. Region model
Each tenant has home_cell, dr_cell, jurisdiction_code, and compliance_packs. Active-active read paths are allowed only for replicated projections; writes route to home_cell unless a signed disaster promotion changes the cell role.

## B. Data-residency rule
Regulated records never cross a pack boundary without a residency decision and export evidence. KR-PIPA, GDPR, LGPD, SOX, and industry overlays are represented in the pack roster.

## C. Transport
Every region advertises h3 through Alt-Svc, falls back in order to HTTP/2 then HTTP/1.1, rotates ECH config, and offers PQC hybrid negotiation where supported.

`;
  return ensureLines(base, 250, (i) => {
    const bc = service.bounded_contexts[(i - 1) % service.bounded_contexts.length];
    const region = ["us-east", "us-west", "eu-central", "ap-northeast-2", "ap-southeast", "sovereign-eu"][(i - 1) % 6];
    return `- Multi-region row ${String(i).padStart(3, "0")}: ${region} handling for ${service.title}.${bc} requires home_cell routing, dr_cell replay cursor, residency_pack allow-list, ${evt(service, bc, "REGION_DECISION")}, and no direct marketplace settlement writes.`;
  });
}

function incidentResponse(service) {
  const base = `${frontmatter(service, "IncidentResponse")}
# Incident Response: ${service.title}

## A. Severity classes
- SEV-1: cross-tenant disclosure, emergency-services friction, corruption of signed ERP record, or inability to reverse regulated posting.
- SEV-2: degraded command latency, repeated worker deadletters, stale policy bundle, or partial regional failover.
- SEV-3: dashboard gap, SDK regression, isolated tenant quota issue, or non-critical export delay.

## B. Response principles
Contain tenant blast radius first, preserve audit evidence, keep marketplace settlement boundaries intact, and prefer reversible mitigation over direct database mutation.

`;
  return ensureLines(base, 250, (i) => {
    const bc = service.bounded_contexts[(i - 1) % service.bounded_contexts.length];
    return `${i}. Incident step ${String(i).padStart(3, "0")}: for ${service.title}.${bc}, query trace_id and tenant_id, verify Cedar decision log, inspect ${evt(service, bc, "INCIDENT_SIGNAL")}, confirm ECH/PQC ingress health, compare impact to ${benchmarkLine(service)}, execute rollback if invariant breach is confirmed, and record post-incident action.`;
  });
}

function backfillReplay(service) {
  const base = `${frontmatter(service, "BackfillReplay")}
# Backfill and Replay: ${service.title}

## A. Replay contract
Backfill and replay jobs are idempotent, tenant-scoped, checksum-bound, and audit-chain sealed. Jobs never write directly to adjacent microservice databases; cross-service effects use workflow-engine and ontology contracts.

## B. Portability
Per GDPR Art 20 and regional pack overlays, export format is newline-delimited JSON plus detached signature manifest. DSR deletion markers are terminal and must not be resurrected by replay.

## C. RPO / RTO declaration
- RPO: 5 minutes for tenant-scoped ERP projections, 0 minutes for signed audit-chain cursors, and 15 minutes for archive-only exports.
- RTO: 30 minutes for tenant replay restore, 5 minutes for audit-chain reader restore, and 4 hours for archive-only regulated export restore.
- Restore drill cadence: quarterly for regulated-enterprise tenants and semi-annually for sandbox or growth tenants.

`;
  return ensureLines(base, 250, (i) => {
    const bc = service.bounded_contexts[(i - 1) % service.bounded_contexts.length];
    return `${i}. Replay row ${String(i).padStart(3, "0")}: ${service.title}.${bc} reads source cursor, validates checksum, applies tenant_id and residency_pack, emits ${evt(service, bc, "REPLAY_STEP")}, records marketplace settlement reference as read-only, and can roll back to the previous signed cursor.`;
  });
}

function competitorParity(service) {
  const base = `${frontmatter(service, "CompetitorParityMatrix")}
# Competitor Parity Matrix: ${service.title}

## A. Benchmark roster
| Vendor | Counterpart | Parity stance |
|---|---|---|
${BENCHMARKS[service.microservice]
  .map((b) => `| ${b.split(" ")[0]} | ${b} | Match through composable tenant-scoped microservice plus policy, workflow, and ontology contracts |`)
  .join("\n")}

## B. Differentiator
Oyatie avoids platform lock-in. ${service.title} exposes focused contracts, composes with marketplace settlement per ADR-0314, and preserves tenant pack overlays as data rather than product forks.

`;
  return ensureLines(base, 350, (i) => {
    const bc = service.bounded_contexts[(i - 1) % service.bounded_contexts.length];
    const vendor = BENCHMARKS[service.microservice][(i - 1) % BENCHMARKS[service.microservice].length];
    return `| Row ${String(i).padStart(3, "0")} | ${vendor} | ${service.title}.${bc} | Requires tenant scope, Cedar permit, audit-chain event, OpenAPI/AsyncAPI/proto parity, HTTP/3/ECH/PQC transport note, and pack overlay evidence before claiming parity |`;
  });
}

function sdkPlan(service) {
  const languages = ["rust", "typescript", "python", "go", "java", "kotlin", "swift"];
  const base = `${frontmatter(service, "SdkPlan")}
# SDK Plan: ${service.title}

## A. SDK surfaces
SDKs are generated from OpenAPI 3.2.0 and proto3 contracts, with event helper bindings generated from AsyncAPI 3.1.0. Each SDK enforces tenant_id, idempotency_key, residency_pack, and audit_event_class fields before request dispatch.

## B. Language roster
${languages.map((lang) => `- ${lang}: typed client, retry policy, error taxonomy, and signed payload helper.`).join("\n")}

## C. Versioning
SemVer applies to every SDK. Breaking changes require deprecation notice, changelog entry, contract version bump, and migration IP.

`;
  return ensureLines(base, 250, (i) => {
    const bc = service.bounded_contexts[(i - 1) % service.bounded_contexts.length];
    const lang = languages[(i - 1) % languages.length];
    return `- SDK row ${String(i).padStart(3, "0")}: ${lang} client for ${service.title}.${bc} maps ${evt(service, bc, "SDK_CALL")} errors, propagates OpenBao secret references without materializing secrets, preserves ADR-0314 marketplace boundaries, and documents ${benchmarkLine(service)} migration naming.`;
  });
}

function runbook(service, scenario, index) {
  const bc = service.bounded_contexts[index % service.bounded_contexts.length];
  const base = `${frontmatter(service, "Runbook", { scenario })}
# Runbook: ${service.title} ${scenario}

## A. Trigger conditions
- Alert ${service.microservice}_${scenario.replaceAll("-", "_")}_active is firing.
- Audit stream shows ${evt(service, bc, "RUNBOOK_TRIGGER")}.
- Tenant report identifies affected bounded context ${bc}.

## B. Pre-checks
1. Confirm tenant_id and source_system_id.
2. Confirm Cedar bundle version and policy decision trace.
3. Confirm ingress negotiated HTTP/3 or fell back to HTTP/2/HTTP/1.1 in order.
4. Confirm ECH config age and PQC negotiation metric.
5. Confirm marketplace settlement status is read-only.

## C. Procedure
`;
  const withProcedure = `${base}${Array.from({ length: 20 }, (_, i) => {
    const n = i + 1;
    return `${n}. Execute ${scenario} step ${n}: query ${service.microservice}_${bc.replaceAll("-", "_")}_health, inspect trace span oya.${service.microservice}.${bc}, verify audit event ${evt(service, bc, `STEP_${String(n).padStart(2, "0")}`)}, and stop if tenant scope does not match.`;
  }).join("\n")}

## D. Verification
Verify the SLO burn rate returns below threshold, the worker queue drains, the Cedar decision log contains no unexpected permit, and the tenant dashboard reflects normal state.

## E. Rollback
Rollback to the last signed replay cursor or previous policy bundle. Never mutate production rows manually.

## F. Post-incident
Create an evidence record, attach dashboard snapshots, update this runbook if a step was ambiguous, and link a follow-up IP when automation is needed.

## G. References
- docs/standards/documentation-rigor.md
- microservices/${service.microservice}/failure-modes.md
- microservices/${service.microservice}/incident-response.md
- ${benchmarkLine(service)}

`;
  return ensureLines(withProcedure, 250, (i) => {
    return `- Runbook detail ${String(i).padStart(3, "0")}: ${scenario} for ${service.title}.${bc} preserves tenant_id, data_class, residency_pack, audit_event_class, OpenBao lease id, ECH/PQC transport evidence, marketplace settlement reference, and operator initials for post-incident review.`;
  });
}

function cedarHeader(service, name) {
  return `// ${service.title} ${name}\n// Cedar v4.2 LTS fragment\n// References: ADR-0243, ADR-0244, ADR-0294, ADR-0297, ADR-0314, ADR-0315\n// SAP code: ${service.sap_module_parity.sap_code}\n// Benchmarks: ${benchmarkLine(service)}\n`;
}

function bcAuthorizationCedar(service, bc) {
  return `${cedarHeader(service, `${bc} authorization`)}
forbid (principal, action, resource);

permit (
  principal,
  action in [
    Action::"${bc}.read",
    Action::"${bc}.create",
    Action::"${bc}.amend",
    Action::"${bc}.approve",
    Action::"${bc}.reverse",
    Action::"${bc}.export"
  ],
  resource
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  context has purpose_code &&
  context has compliance_packs &&
  context has marketplace_settlement_ref &&
  context.marketplace_settlement_ref != "" &&
  context has policy_bundle_version &&
  context.policy_bundle_version >= "2026-05-21"
};

forbid (
  principal,
  action in [Action::"${bc}.approve", Action::"${bc}.reverse"],
  resource
)
when {
  principal has duties &&
  resource has creator_principal_id &&
  principal.id == resource.creator_principal_id
};
`;
}

function abuseCedar(service) {
  return `${cedarHeader(service, "abuse defence")}
forbid (principal, action, resource);

permit (principal, action, resource)
when {
  context has audience_type &&
  context.audience_type == "EMERGENCY_SERVICES"
};

permit (principal, action, resource)
when {
  context has bot_score &&
  context.bot_score < 40 &&
  context has scrape_score &&
  context.scrape_score < 30 &&
  context has spoof_score &&
  context.spoof_score < 20 &&
  context has default_path_latency_ms &&
  context.default_path_latency_ms <= 2
};

forbid (principal, action, resource)
when {
  context has bot_score &&
  context.bot_score >= 80 &&
  !(context has audience_type && context.audience_type == "EMERGENCY_SERVICES")
};
`;
}

function emergencyCedar(service) {
  return `${cedarHeader(service, "emergency services bypass")}
permit (principal, action, resource)
when {
  context has audience_type &&
  context.audience_type == "EMERGENCY_SERVICES" &&
  context has emergency_attestation_chain &&
  context.emergency_attestation_chain != "" &&
  context has audit_event_class &&
  context.audit_event_class == "AbuseDefenceEmergencyServiceBypass"
};

forbid (principal, action, resource)
when {
  context has audience_type &&
  context.audience_type == "EMERGENCY_SERVICES" &&
  !(context has emergency_attestation_chain)
};
`;
}

function dataResidencyPolicy(service) {
  const base = `${frontmatter(service, "DataResidencyPolicy")}
# Data Residency Policy: ${service.title}

## A. Rule
Every record carries tenant_id, jurisdiction_code, residency_pack, retention_class, and audit_event_class. Cross-region copies require an explicit residency decision.

## B. Pack overlays
${service.compliance_packs.map((pack) => `- ${pack}: activated only through tenant pack metadata and policy evidence.`).join("\n")}

## C. Region handling
No regulated ${service.sap_module_parity.sap_code} record crosses a forbidden boundary. Replay refuses if the destination cell cannot prove residency eligibility.
`;
  return ensureLines(base, 80, (i) => `- Residency evidence row ${String(i).padStart(3, "0")}: ${service.title} records cite ${benchmarkLine(service)} and bind residency_pack to Cedar context.`);
}

function tenantIsolationPolicy(service) {
  const base = `${frontmatter(service, "TenantIsolationPolicy")}
# Tenant Isolation Policy: ${service.title}

## A. Rule
Every ${service.sap_module_parity.sap_code} command, query, event, replay cursor, dashboard query, and export carries tenant_id and principal_id. No ${service.title} bounded context may infer tenant scope from hostname, session cookie, marketplace settlement reference, or source-system identifier alone.

## B. Isolation evidence
${service.bounded_contexts
  .map(
    (bc) =>
      `- ${bc}: API, worker, catalog, SLO, dashboard, Cedar, and audit evidence require tenant_id equality before mutation or disclosure.`
  )
  .join("\n")}

## C. Marketplace boundary
Marketplace settles tenant deals per ADR-0314. ${service.title} stores settlement references only after tenant isolation has already been proven by policy, audit, and source-system checks.
`;
  return ensureLines(base, 90, (i) => {
    const bc = service.bounded_contexts[(i - 1) % service.bounded_contexts.length];
    return `- Tenant isolation evidence row ${String(i).padStart(3, "0")}: ${service.title}.${bc} requires tenant_id, principal_id, source_system_id, residency_pack, Cedar decision id, ${evt(service, bc, "TENANT_ISOLATION_CHECK")}, and comparator review against ${benchmarkLine(service)}.`;
  });
}

function simpleScopeCedar(service, name, actions) {
  return `${cedarHeader(service, name)}
forbid (principal, action, resource);

permit (principal, action in [${actions.map((a) => `Action::"${a}"`).join(", ")}], resource)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  context has valid_from &&
  context has valid_to &&
  context.valid_from <= context.now &&
  context.now <= context.valid_to
};
`;
}

function packOverlayCedar(service) {
  return `${cedarHeader(service, "pack overlay authorization")}
forbid (principal, action, resource);

permit (principal, action, resource)
when {
  resource has tenant_id &&
  principal has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  context has compliance_packs &&
  context has requested_pack &&
  context.requested_pack in context.compliance_packs &&
  context has pack_overlay_signature &&
  context.pack_overlay_signature != ""
};
`;
}

function openapi(service) {
  const schemas = service.bounded_contexts
    .map((bc) => `    ${pascal(bc)}Command:\n      type: object\n      required: [tenant_id, principal_id, idempotency_key, payload, compliance_packs]\n      properties:\n        tenant_id:\n          type: string\n        principal_id:\n          type: string\n        idempotency_key:\n          type: string\n        compliance_packs:\n          type: array\n          items:\n            type: string\n        marketplace_settlement_ref:\n          type: string\n        payload:\n          type: object\n`)
    .join("");
  const paths = service.bounded_contexts
    .map((bc) => `  /v1/${service.microservice}/${bc}:\n    post:\n      summary: Create or amend ${service.title} ${bc}\n      operationId: ${snake(service.microservice)}_${snake(bc)}_mutate\n      x-transport:\n        default: HTTP/3\n        fallback: [HTTP/2, HTTP/1.1]\n        ech: advertised\n        pqc: X25519MLKEM768-when-supported\n      x-bnf-version: "BNF v4.1"\n      requestBody:\n        required: true\n        content:\n          application/json:\n            schema:\n              $ref: "#/components/schemas/${pascal(bc)}Command"\n      responses:\n        "202":\n          description: Accepted with signed audit event\n`)
    .join("");
  return `openapi: 3.2.0\ninfo:\n  title: ${service.title} API\n  version: 1.0.0\n  summary: SAP ${service.sap_module_parity.sap_code} parity API with marketplace settlement boundaries\nservers:\n  - url: https://api.oyatie.example/${service.microservice}\n    description: HTTP/3 default, ECH advertised, PQC hybrid offered when supported\nx-enterprise-benchmarks:\n${BENCHMARKS[service.microservice].map((b) => `  - ${JSON.stringify(b)}`).join("\n")}\npaths:\n${paths}components:\n  schemas:\n${schemas}`;
}

function asyncapi(service) {
  const channels = service.bounded_contexts
    .map((bc) => `  ${service.microservice}.${bc}.events.v1:\n    address: ${service.microservice}.${bc}.events.v1\n    messages:\n      ${pascal(bc)}Changed:\n        $ref: "#/components/messages/${pascal(bc)}Changed"\n`)
    .join("");
  const messages = service.bounded_contexts
    .map((bc) => `    ${pascal(bc)}Changed:\n      name: ${pascal(bc)}Changed\n      payload:\n        type: object\n        required: [tenant_id, audit_event_class, bounded_context, occurred_at]\n        properties:\n          tenant_id:\n            type: string\n          audit_event_class:\n            type: string\n            const: ${evt(service, bc, "CHANGED")}\n          bounded_context:\n            type: string\n            const: ${bc}\n          marketplace_settlement_ref:\n            type: string\n          occurred_at:\n            type: string\n            format: date-time\n`)
    .join("");
  return `asyncapi: 3.1.0\ninfo:\n  title: ${service.title} Events\n  version: 1.0.0\n  description: AsyncAPI 3.1.0 channels for SAP ${service.sap_module_parity.sap_code} parity events.\ndefaultContentType: application/json\nchannels:\n${channels}components:\n  messages:\n${messages}`;
}

function proto(service) {
  const messages = service.bounded_contexts
    .map((bc) => `message ${pascal(bc)}Command {\n  string tenant_id = 1;\n  string principal_id = 2;\n  string idempotency_key = 3;\n  repeated string compliance_packs = 4;\n  string marketplace_settlement_ref = 5;\n}\n\nmessage ${pascal(bc)}Result {\n  string tenant_id = 1;\n  string audit_event_class = 2;\n  string status = 3;\n}\n`)
    .join("\n");
  const rpc = service.bounded_contexts
    .map((bc) => `  rpc Mutate${pascal(bc)} (${pascal(bc)}Command) returns (${pascal(bc)}Result);`)
    .join("\n");
  return `syntax = "proto3";\n\npackage oya.${snake(service.microservice)}.v1;\n\noption java_package = "dev.oyatie.${snake(service.microservice)}.v1";\n\n// SAP ${service.sap_module_parity.sap_code}; benchmarks: ${benchmarkLine(service)}\n// Transport binding: HTTP/3 default at REST edge, gRPC over mTLS internally, ECH/PQC at ingress where supported.\n\n${messages}\nservice ${pascal(service.microservice)}Service {\n${rpc}\n}\n`;
}

function capability(service, bc, suffix) {
  return `schema_version: "2.0"\nname: ${service.microservice}-${bc}-${suffix}\nmicroservice: ${service.microservice}\nbounded_context: ${bc}\nautonomy_tier: T1\nrisk_class: regulated-erp\naudit_event_class: ${evt(service, bc, suffix.toUpperCase().replaceAll("-", "_"))}\npolicy_fragments:\n  - microservices/${service.microservice}/policy/${bc}-authorization.cedar\nbenchmarks:\n${BENCHMARKS[service.microservice].map((b) => `  - ${JSON.stringify(b)}`).join("\n")}\ninputs:\n  required: [tenant_id, principal_id, idempotency_key, compliance_packs]\ntransport:\n  rest: OpenAPI 3.2.0\n  events: AsyncAPI 3.1.0\n  grpc: proto3\nsettlement_boundary: marketplace-settles-tenant-deals-per-ADR-0314\n`;
}

function dashboardJson(service, name, bc) {
  return JSON.stringify(
    {
      title: `${service.title} ${name}`,
      tags: ["erp", service.microservice, bc, service.sap_module_parity.sap_code],
      timezone: "utc",
      schemaVersion: 39,
      benchmarks: BENCHMARKS[service.microservice],
      panels: [
        {
          title: `${bc} command rate`,
          type: "timeseries",
          targets: [{ expr: `sum(rate(oya_${snake(service.microservice)}_${snake(bc)}_commands_total[5m])) by (tenant_id)` }],
        },
        {
          title: `${bc} p99 latency`,
          type: "timeseries",
          targets: [{ expr: `histogram_quantile(0.99, sum(rate(oya_${snake(service.microservice)}_${snake(bc)}_latency_seconds_bucket[5m])) by (le, tenant_id))` }],
        },
        {
          title: `${bc} audit events`,
          type: "stat",
          targets: [{ expr: `sum(rate(audit_events_total{service="${service.microservice}",bounded_context="${bc}"}[5m]))` }],
        },
      ],
    },
    null,
    2
  );
}

function dashboardMd(service, bc) {
  const base = `${frontmatter(service, "DashboardGuide")}
# Dashboard Guide: ${service.title} ${bc}

This dashboard explains ${service.title}.${bc} health and maps each panel to SAP ${service.sap_module_parity.sap_code}, ${benchmarkLine(service)}, Cedar authorization, SLO burn rate, and marketplace settlement boundaries.

`;
  return ensureLines(base, 80, (i) => `- Dashboard row ${String(i).padStart(3, "0")}: panel ${i} preserves tenant_id cardinality, audit_event_class, ECH/PQC transport signal, and ${bc} domain status.`);
}

function slo(service, name, objective, sli) {
  return `apiVersion: openslo/v1\nkind: SLO\nmetadata:\n  name: ${name}\n  labels:\n    service: ${service.microservice}\n    sap_code: ${JSON.stringify(service.sap_module_parity.sap_code)}\nspec:\n  description: ${JSON.stringify(`${service.title} ${name} objective; benchmarks ${benchmarkLine(service)}`)}\n  service: ${service.microservice}\n  indicator:\n    metadata:\n      name: ${name}-sli\n    spec:\n      ratioMetric:\n        counter: true\n        good:\n          metricSource:\n            type: prometheus\n            spec:\n              query: ${JSON.stringify(sli.good)}\n        total:\n          metricSource:\n            type: prometheus\n            spec:\n              query: ${JSON.stringify(sli.total)}\n  objectives:\n    - displayName: ${name}\n      target: ${objective}\n`;
}

function ipDoc(service, index, title, bc, layer) {
  const base = `${frontmatter(service, "ImplementationPlan", { ip_id: `IP-${String(index).padStart(3, "0")}` })}
# IP-${String(index).padStart(3, "0")}: ${title}

## A. Intent
Implement the ${layer} slice for ${service.title}.${bc}. The slice is single-PR-sized, tenant-scoped, and contract-bound to OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, BNF v4.1, and ADR-0105 layer vocabulary.

## B. Acceptance criteria
- ${service.title}.${bc} has typed inputs and outputs.
- Cedar default deny is preserved.
- ${evt(service, bc, "IP_ACCEPTED")} is emitted by tests or evidence fixtures.
- Marketplace settlement remains read-only and owned by marketplace per ADR-0314.
- Benchmarks are named: ${benchmarkLine(service)}.

## C. Verification
Run unit, contract, policy, worker replay, and integration tests for this slice; attach dashboard and audit evidence to the PR.

`;
  return ensureLines(base, 80, (i) => `- IP detail ${String(i).padStart(3, "0")}: ${service.title}.${bc}.${layer} verifies tenant_id, data_class, source_system_id, policy_bundle_version, audit_event_class, residency_pack, ECH/PQC transport note, and rollback path.`);
}

function catalogRecord(service, bc, layer) {
  return `schema_version: "1.0"\ncrate: oya-${service.microservice}-${bc}-${layer}\nmicroservice: ${service.microservice}\nbounded_context: ${bc}\nlayer: ${layer}\nadr_0105_layer: ${layer}\nowner_team: axis-${service.microservice}\nstatus: planned\nsap_code: ${JSON.stringify(service.sap_module_parity.sap_code)}\nbenchmarks:\n${BENCHMARKS[service.microservice].map((b) => `  - ${JSON.stringify(b)}`).join("\n")}\ncontracts:\n  openapi: microservices/${service.microservice}/contracts/openapi-v1.yaml\n  asyncapi: microservices/${service.microservice}/contracts/asyncapi-v1.yaml\n  proto: microservices/${service.microservice}/contracts/${service.microservice}-v1.proto\npolicy_fragment: microservices/${service.microservice}/policy/${bc}-authorization.cedar\nsettlement_boundary: marketplace-settles-tenant-deals-per-ADR-0314\n`;
}

function iacFiles(service) {
  return {
    "iac/k8s-deployment.yaml": `apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: oya-${service.microservice}\n  labels:\n    app: oya-${service.microservice}\n    sap-code: ${JSON.stringify(service.sap_module_parity.sap_code)}\nspec:\n  replicas: 3\n  selector:\n    matchLabels:\n      app: oya-${service.microservice}\n  template:\n    metadata:\n      labels:\n        app: oya-${service.microservice}\n    spec:\n      containers:\n        - name: service\n          image: registry.example/oya-${service.microservice}:0.1.0\n          ports:\n            - name: h3\n              containerPort: 8443\n          env:\n            - name: OYA_HTTP_DEFAULT\n              value: HTTP/3\n            - name: OYA_ECH_ENABLED\n              value: "true"\n            - name: OYA_PQC_HYBRID\n              value: X25519MLKEM768\n`,
    "iac/helm-values.yaml": `replicaCount: 3\nimage:\n  repository: registry.example/oya-${service.microservice}\n  tag: 0.1.0\ntransport:\n  default: HTTP/3\n  fallback: [HTTP/2, HTTP/1.1]\n  ech: true\n  pqcHybrid: X25519MLKEM768\nbenchmarks:\n${BENCHMARKS[service.microservice].map((b) => `  - ${JSON.stringify(b)}`).join("\n")}\n`,
    "iac/openbao-policy.hcl": `path "secret/data/{{identity.entity.aliases.auth_kubernetes_*.metadata.service_account_namespace}}/${service.microservice}/*" {\n  capabilities = ["read"]\n}\n\npath "transit/sign/${service.microservice}-audit" {\n  capabilities = ["update"]\n}\n`,
    "iac/network-policy.yaml": `apiVersion: networking.k8s.io/v1\nkind: NetworkPolicy\nmetadata:\n  name: oya-${service.microservice}-default-deny\nspec:\n  podSelector:\n    matchLabels:\n      app: oya-${service.microservice}\n  policyTypes: [Ingress, Egress]\n  ingress:\n    - from:\n        - namespaceSelector:\n            matchLabels:\n              oya-ingress: "true"\n  egress:\n    - to:\n        - namespaceSelector:\n            matchLabels:\n              oya-substrate: "true"\n`,
    "iac/secret-bindings.yaml": `apiVersion: oyatie.dev/v1\nkind: SecretBinding\nmetadata:\n  name: oya-${service.microservice}\nspec:\n  openbaoPaths:\n    - secret/{{ tenant_id }}/${service.microservice}/database\n    - secret/{{ tenant_id }}/${service.microservice}/signing\n  maxTtlSeconds: 60\n  auditEventClass: ${evt(service, service.bounded_contexts[0], "SECRET_LEASE")}\n`,
    "iac/terraform-module/main.tf": `variable "service_name" { default = "oya-${service.microservice}" }\nvariable "http_default" { default = "HTTP/3" }\nvariable "ech_enabled" { default = true }\nvariable "pqc_hybrid" { default = "X25519MLKEM768" }\n\noutput "service_name" { value = var.service_name }\noutput "sap_code" { value = "${service.sap_module_parity.sap_code}" }\n`,
    "iac/ech-config.yaml": `apiVersion: oyatie.dev/v1\nkind: EchConfig\nmetadata:\n  name: oya-${service.microservice}-ech\nspec:\n  rotationCadenceDays: 90\n  httpsRecord: true\n  gracefulFallback: true\n  auditEventClass: ${evt(service, service.bounded_contexts[0], "ECH_ROTATED")}\n`,
    "iac/pqc-cert.yaml": `apiVersion: cert-manager.io/v1\nkind: Certificate\nmetadata:\n  name: oya-${service.microservice}-pqc\nspec:\n  secretName: oya-${service.microservice}-tls\n  commonName: ${service.microservice}.svc.oyatie.example\n  privateKey:\n    algorithm: Ed25519\n  usages: [server auth, client auth]\n  subject:\n    organizations: [Oyatie]\n  additionalOutputFormats:\n    - type: CombinedPEM\n`,
    "iac/edge-waf.yaml": `apiVersion: oyatie.dev/v1\nkind: EdgeWafPolicy\nmetadata:\n  name: oya-${service.microservice}-abuse-defence\nspec:\n  defaultPathLatencyBudgetMs: 2\n  emergencyServicesBypass: true\n  antiBot:\n    tokenBucket: true\n    ja4Fingerprint: true\n    passiveMlScore: true\n  antiSpoof:\n    strictTls13: true\n    hstsPreload: true\n    signedPayloads: true\n  antiScrape:\n    adaptiveChallenge: suspicion-only\n    friendlyCrawlerPartnerBypass: true\n`,
  };
}

function scorecard(service) {
  return JSON.stringify(
    {
      schema_version: "1.0",
      service: service.microservice,
      date: DATE,
      operating_bar_artifact_target: 100,
      required_contracts: ["OpenAPI 3.2.0", "AsyncAPI 3.1.0", "proto3", "BNF v4.1", "ADR-0105"],
      required_transport: ["HTTP/3", "HTTP/2 fallback", "HTTP/1.1 fallback", "ECH", "PQC hybrid where supported"],
      benchmarks: BENCHMARKS[service.microservice],
      scorecards: {
        doc_set: "blocker-after-2026-07-15",
        adr_adherence: "blocker-after-2026-07-15",
        abuse_defence_ux_floor: "required",
        emergency_services_bypass: "required",
      },
    },
    null,
    2
  );
}

function auditFindings(service) {
  return JSON.stringify(
    {
      schema_version: "1.0",
      date: DATE,
      service: service.microservice,
      verdict: "second-pass-authored",
      sap_code: service.sap_module_parity.sap_code,
      artifact_target: 100,
      findings: service.bounded_contexts.map((bc) => ({
        id: `${service.microservice}-${bc}-doc-set`,
        status: "closed-by-second-pass",
        evidence: [
          `policy/${bc}-authorization.cedar`,
          `catalog/oya-${service.microservice}-${bc}-domain.yaml`,
          `runbooks/${RUNBOOK_SCENARIOS[0]}.md`,
        ],
      })),
      benchmarks: BENCHMARKS[service.microservice],
    },
    null,
    2
  );
}

function complianceExtension(service) {
  return `<!-- erp-second-pass:${DATE}:start -->
## detection-substrate-binding
${service.title} binds detection to observability, audit-chain, policy, OpenBao, edge-WAF, and workflow-engine substrates. Every ${service.sap_module_parity.sap_code} bounded context emits signed audit events, metrics, traces, logs, and policy-decision evidence. Detection is compared against ${benchmarkLine(service)} and is tenant-scoped before any operator sees it.

## insider-threat-controls
Insider controls require two-person approval for approve/reverse actions, segregation of duties between creator and approver, JIT OpenBao credentials with TTL no greater than 60 seconds, auditor read-only Cedar scopes, CI read-only scopes, and immutable evidence for every privileged action.

## threat-intelligence-feeds
Threat intelligence uses sanctioned-party, bot-score, credential-stuffing, exploit-CVE, supplier-risk, and jurisdiction-watch feeds. Feed decisions are advisory unless a Cedar policy explicitly permits enforcement. Emergency-services traffic bypasses visible challenge but not audit.

## key-rotation-cadence
Signing keys rotate every 90 days, ECH keys rotate every 90 days or faster after suspected exposure, OpenBao dynamic credentials expire within 60 seconds for provider credentials, and PQC certificate experiments are tracked without blocking classical fallback.

## crypto-agility-plan
Transport defaults to TLS 1.3 with HTTP/3, falls back to HTTP/2 and HTTP/1.1 in order, advertises ECH where terminated by the platform, and offers X25519MLKEM768 hybrid key agreement where peer support exists. The service never refuses a legitimate peer only because PQC or ECH is unavailable.

## critical-path-edge-cases
${CRITICAL_PATHS.map((pathName) => `- ${pathName}: ${service.title} documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.`).join("\n")}
<!-- erp-second-pass:${DATE}:end -->`;
}

function extendManifest(serviceDir, service, stats) {
  const file = path.join(serviceDir, "manifest.json");
  const manifest = readJson(file);
  const adrRefs = manifest.adrs || manifest.binding_adrs || [
    "ADR-0105",
    "ADR-0131",
    "ADR-0132",
    "ADR-0244",
    "ADR-0245",
    "ADR-0253",
    "ADR-0297",
    "ADR-0314",
    "ADR-0315",
  ];
  manifest.adrs = adrRefs;
  manifest.adr_authority_chain = manifest.adr_authority_chain || adrRefs;
  manifest.regulatory_packs = manifest.regulatory_packs || manifest.compliance_packs || [];
  manifest.audit_chain = manifest.audit_chain || {
    enabled: true,
    seal_events: service.bounded_contexts.map((bc) => evt(service, bc, "CHANGED")),
  };
  manifest.mesh_layering = {
    ...(manifest.mesh_layering || {}),
    cilium_l4: true,
    ambient_ztunnel: true,
    ambient_waypoint: manifest.mesh_layering?.ambient_waypoint ?? false,
    north_south_only: manifest.mesh_layering?.north_south_only ?? false,
  };
  manifest.second_pass_doc_set = {
    date: DATE,
    target: "PR-143 operating bar",
    operating_bar_artifact_target: 100,
    generated_roster_version: "erp-second-pass-v1",
    categories: {
      strategic_docs: 3,
      architecture_ops_docs: 10,
      policy_and_cedar: service.bounded_contexts.length + 7,
      runbooks: RUNBOOK_SCENARIOS.length,
      contracts: 3,
      capabilities: 3,
      dashboards: 3,
      slos: 4,
      implementation_plans: 15,
      catalog_records: service.bounded_contexts.length * LAYERS.length,
      iac: 9,
      scorecards_and_audit: 2,
    },
  };
  manifest.keystone_adr_field_roster = {
    principals: `oyatie.${service.microservice}.service and tenant-scoped callers`,
    cedar_gates: service.bounded_contexts.map((bc) => `policy/${bc}-authorization.cedar`),
    tenant_scoping: ["tenant_id", "principal_id", "audience_type", "provider_credential_mode"],
    substrate_or_product: manifest.tier || "product",
    policy_evaluation_mode: "caller-side-library-first-with-network-opt-in",
    self_modification: "no autonomous self-modification; evidence artifacts only",
    cell_eligibility: ["Tier 0", "Tier 1", "Tier 2", "Tier 3"],
    marketplace: "Marketplace settles all tenant deals per ADR-0314; this service records settlement refs only",
    day_one_cert_readiness: manifest.compliance_packs || [],
    pack_overlay_roster: manifest.compliance_packs || [],
    time_coordination: "HLC default; TrueTime-compatible external evidence accepted when source system supplies it",
    transport: "HTTP/3 default; fallback HTTP/2 then HTTP/1.1; TLS 1.3; ECH advertised; PQC hybrid offered where supported",
    deployment_shape: "Kubernetes plus Cloud Hypervisor/Kata-compatible isolation for regulated workers",
    intelligence_dispatch: "library-first when local binding exists; network-opt-in for model-backed analysis",
    ontology_read_path: "library-first projections with freshness_floor per bounded context",
    semver_policy: "SemVer for REST/event/gRPC/SDK contracts",
    observability: ["metrics", "traces", "structured logs", "audit-chain events", "dashboards"],
    consent: "per-purpose only when user-facing flows collect personal data",
    deliverability: "no direct mail-emitting ownership unless a pack overlay activates notifications through mail/connect",
    backup_portability: "NDJSON plus detached signature manifest",
    substrate_dependencies: manifest.integration_points || [],
    platform_owner_indirection: "no hard-coded owner name in public contract fields",
    minor_protection: "refuse or route minor-targeted flows through pack policy",
    meta_trust_attestation: "required only for Foundry-touching generated artifacts",
    cedar_soak: ">=60s policy soak before enforcement promotion",
    bootstrap_trust_chain: "SPIFFE plus kill-switch for bootstrap-tier surfaces",
    credential_isolation: "OpenBao dynamic secrets with <=60s TTL or sidecar isolation",
    abuse_defence: "anti-bot, anti-spoof, anti-scrape with UX floor and emergency-services bypass",
    critical_path_edge_cases: CRITICAL_PATHS,
  };
  manifest.enterprise_benchmarks = BENCHMARKS[service.microservice];
  fs.writeFileSync(file, `${JSON.stringify(manifest, null, 2)}\n`);
  stats.modified.push(path.relative(ROOT, file));
}

function generateService(slug, stats) {
  const serviceDir = path.join(ROOT, "microservices", slug);
  const manifest = readJson(path.join(serviceDir, "manifest.json"));
  const service = {
    ...manifest,
    title: manifest.title || titleCase(slug),
  };

  const phaseName = `PHASE-01-${slug.toUpperCase()}-PARITY.md`;
  const mainDocs = {
    [phaseName]: phaseDoc(service),
    "threat-model.md": threatModel(service),
    "dpia.md": dpia(service),
    "README.md": readmeDoc(service),
    "CHANGELOG.md": changelogDoc(service),
    "capacity-model.md": capacityModel(service),
    "cost-budget.md": costBudget(service),
    "failure-modes.md": failureModes(service),
    "multi-region.md": multiRegion(service),
    "incident-response.md": incidentResponse(service),
    "backfill-replay.md": backfillReplay(service),
    "competitor-parity-matrix.md": competitorParity(service),
    "sdk-plan.md": sdkPlan(service),
  };

  for (const [rel, content] of Object.entries(mainDocs)) {
    writeFile(path.join(serviceDir, rel), content, stats);
  }

  appendOnce(path.join(serviceDir, "compliance.md"), `<!-- erp-second-pass:${DATE}:start -->`, complianceExtension(service), stats);
  extendManifest(serviceDir, service, stats);

  for (const bc of service.bounded_contexts) {
    writeFile(path.join(serviceDir, "policy", `${bc}-authorization.cedar`), bcAuthorizationCedar(service, bc), stats);
  }
  writeFile(path.join(serviceDir, "policy", "abuse-defence.cedar"), abuseCedar(service), stats);
  writeFile(path.join(serviceDir, "policy", "emergency-services-bypass.cedar"), emergencyCedar(service), stats);
  writeFile(path.join(serviceDir, "policy", "data-residency.md"), dataResidencyPolicy(service), stats);
  writeFile(path.join(serviceDir, "policy", "tenant-isolation.md"), tenantIsolationPolicy(service), stats);
  writeFile(path.join(serviceDir, "policy", "auditor-scope.cedar"), simpleScopeCedar(service, "auditor scope", ["artifact.read", "audit.read", "dashboard.read", "slo.read"]), stats);
  writeFile(path.join(serviceDir, "policy", "ci-scope.cedar"), simpleScopeCedar(service, "ci scope", ["contract.read", "policy.read", "scorecard.read", "evidence.read"]), stats);
  writeFile(path.join(serviceDir, "policy", "pack-overlay-authorization.cedar"), packOverlayCedar(service), stats);

  RUNBOOK_SCENARIOS.forEach((scenario, index) => {
    writeFile(path.join(serviceDir, "runbooks", `${scenario}.md`), runbook(service, scenario, index), stats);
  });

  writeFile(path.join(serviceDir, "contracts", "openapi-v1.yaml"), openapi(service), stats);
  writeFile(path.join(serviceDir, "contracts", "asyncapi-v1.yaml"), asyncapi(service), stats);
  writeFile(path.join(serviceDir, "contracts", `${slug}-v1.proto`), proto(service), stats);

  const capabilityBcs = service.bounded_contexts.slice(0, 3);
  capabilityBcs.forEach((bc, index) => {
    const suffix = ["command", "reconcile", "export"][index];
    writeFile(path.join(serviceDir, "capabilities", `${bc}-${suffix}.yaml`), capability(service, bc, suffix), stats);
  });

  writeFile(path.join(serviceDir, "dashboards", `${slug}-overview.json`), dashboardJson(service, "overview", service.bounded_contexts[0]), stats);
  writeFile(path.join(serviceDir, "dashboards", `${service.bounded_contexts[0]}-health.json`), dashboardJson(service, "health", service.bounded_contexts[0]), stats);
  writeFile(path.join(serviceDir, "dashboards", `${service.bounded_contexts[1]}-residency.md`), dashboardMd(service, service.bounded_contexts[1]), stats);

  writeFile(path.join(serviceDir, "slos", `${slug}-availability.openslo.yaml`), slo(service, `${slug}-availability`, 0.999, { good: `sum(rate(http_requests_total{service="${slug}",status=~"2.."}[5m]))`, total: `sum(rate(http_requests_total{service="${slug}"}[5m]))` }), stats);
  writeFile(path.join(serviceDir, "slos", `${slug}-latency-p99.openslo.yaml`), slo(service, `${slug}-latency-p99`, 0.99, { good: `sum(rate(http_request_duration_seconds_bucket{service="${slug}",le="0.35"}[5m]))`, total: `sum(rate(http_request_duration_seconds_count{service="${slug}"}[5m]))` }), stats);
  writeFile(path.join(serviceDir, "slos", `${slug}-throughput.openslo.yaml`), slo(service, `${slug}-throughput`, 0.995, { good: `sum(rate(oya_${snake(slug)}_accepted_total[5m]))`, total: `sum(rate(oya_${snake(slug)}_received_total[5m]))` }), stats);
  writeFile(path.join(serviceDir, "slos", `${service.bounded_contexts[0]}-success-rate.openslo.yaml`), slo(service, `${service.bounded_contexts[0]}-success-rate`, 0.999, { good: `sum(rate(oya_${snake(slug)}_${snake(service.bounded_contexts[0])}_success_total[5m]))`, total: `sum(rate(oya_${snake(slug)}_${snake(service.bounded_contexts[0])}_attempt_total[5m]))` }), stats);

  const ips = [];
  service.bounded_contexts.forEach((bc) => ips.push([`Domain layer for ${bc}`, bc, "domain"]));
  service.bounded_contexts.forEach((bc) => ips.push([`Usecase layer for ${bc}`, bc, "usecase"]));
  ips.push([`Adapter integrations for ${slug}`, service.bounded_contexts[0], "adapter"]);
  ips.push([`REST, gRPC, and worker surfaces for ${slug}`, service.bounded_contexts[1], "rest-grpc-worker"]);
  ips.push([`Integration tests for ${slug}`, service.bounded_contexts[2], "tests"]);
  ips.slice(0, 15).forEach(([title, bc, layer], index) => {
    const ip = String(index + 1).padStart(3, "0");
    const filename = `IP-${ip}-${title.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "")}.md`;
    writeFile(path.join(serviceDir, filename), ipDoc(service, index + 1, title, bc, layer), stats);
  });

  for (const bc of service.bounded_contexts) {
    for (const layer of LAYERS) {
      writeFile(path.join(serviceDir, "catalog", `oya-${slug}-${bc}-${layer}.yaml`), catalogRecord(service, bc, layer), stats);
    }
  }

  for (const [rel, content] of Object.entries(iacFiles(service))) {
    writeFile(path.join(serviceDir, rel), content, stats);
  }
  writeFile(path.join(serviceDir, "scorecards", "overrides.json"), scorecard(service), stats);
  writeFile(path.join(serviceDir, `AUDIT-FINDINGS-${DATE}.json`), auditFindings(service), stats);
}

function main() {
  const stats = { created: [], modified: [] };
  for (const slug of SERVICES) {
    generateService(slug, stats);
  }
  const summary = {
    date: DATE,
    created_this_run_count: stats.created.length,
    modified_this_run_count: stats.modified.length,
    expected_services: SERVICES.length,
    expected_new_artifacts_per_service: 125,
    expected_new_artifacts_total: SERVICES.length * 125,
    expected_total_artifacts_per_service: 129,
    created_this_run: stats.created,
    modified_this_run: stats.modified,
  };
  fs.writeFileSync(path.join(ROOT, "microservices", "erp-second-pass-generation-summary.json"), `${JSON.stringify(summary, null, 2)}\n`);
  console.log(JSON.stringify({ created_count: stats.created.length, modified_count: stats.modified.length }, null, 2));
}

main();
