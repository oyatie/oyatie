// TypeScript port of crates/oya-application-shell-frontend-prototype/src/render_envelope.rs
// Mirrors the Rust structs 1-to-1 so the /api/render-envelope/:context JSON
// response can be deserialized without a codegen step for this internal shape.
// When the Rust backend publishes an OpenAPI contract for this endpoint, replace
// this hand-maintained file with the generated client from `npm run codegen`.

export type DemoContext = "tenant-admin" | "corporate-office" | "healthcare-clinician";

export const DEMO_CONTEXTS: DemoContext[] = [
  "tenant-admin",
  "corporate-office",
  "healthcare-clinician",
];

export const DEMO_CONTEXT_LABELS: Record<DemoContext, string> = {
  "tenant-admin": "Tenant admin",
  "corporate-office": "Corporate office",
  "healthcare-clinician": "Accredited healthcare",
};

export const DEMO_CONTEXT_ROLES: Record<DemoContext, string> = {
  "tenant-admin": "Cloud owner / tenant admin",
  "corporate-office": "Accounting + HR operations user",
  "healthcare-clinician": "Clinician in accredited healthcare tenant",
};

export interface AccreditationState {
  label: string;
  healthcare_enabled: boolean;
  explanation: string;
}

export interface MetricCard {
  label: string;
  value: string;
  detail: string;
}

export interface ModuleCard {
  name: string;
  group: string;
  description: string;
  action_label: string;
}

export interface WorkItem {
  title: string;
  detail: string;
  priority: string;
}

export interface ScheduleItem {
  time: string;
  title: string;
  detail: string;
}

export interface MessageItem {
  from: string;
  channel: string;
  preview: string;
}

export interface CommunityItem {
  space: string;
  topic: string;
  activity: string;
}

export interface ApprovalItem {
  title: string;
  requester: string;
  risk_note: string;
}

export interface WorkflowNode {
  id: string;
  label: string;
  kind: string;
  x: number;
  y: number;
  explanation: string;
}

export interface WorkflowEdge {
  from: string;
  to: string;
  label: string;
}

export interface WorkflowPreview {
  name: string;
  goal: string;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
}

export interface OntologyFact {
  entity: string;
  relation: string;
  access_reason: string;
}

export interface IntelligenceSuggestion {
  title: string;
  body: string;
  guardrail: string;
}

export interface ProductActivityStep {
  route_key: string;
  label: string;
  surface: string;
  detail: string;
  target: string;
  state: string;
}

export interface ProductActivitySpine {
  active_route: string;
  active_context: string;
  status_label: string;
  evidence_id: string;
  steps: ProductActivityStep[];
}

export interface TenantRenderEnvelope {
  context: DemoContext;
  tenant_name: string;
  role_name: string;
  tenant_class: string;
  accreditation: AccreditationState;
  server_derivation_note: string;
  product_activity: ProductActivitySpine;
  metrics: MetricCard[];
  modules: ModuleCard[];
  daily_tasks: WorkItem[];
  schedule: ScheduleItem[];
  messages: MessageItem[];
  community: CommunityItem[];
  approvals: ApprovalItem[];
  workflow: WorkflowPreview;
  ontology: OntologyFact[];
  intelligence: IntelligenceSuggestion[];
  omitted_capability_note: string;
}
