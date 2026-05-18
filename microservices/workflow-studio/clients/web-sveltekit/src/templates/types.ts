// Workflow Studio template catalog — shared types.
// Mirrors microservices/workflow-studio/templates/schemas/workflow-template.schema.json
// (only the projection exposed via index.json — full WorkflowStudioTemplate is
// loaded on-demand from definition_path when a user opens the detail pane).

export type Vertical =
  | "hr-people"
  | "payroll-finance"
  | "operations"
  | "hospital-operations"
  | "hiring";

export type Persona =
  | "hr-business-partner"
  | "people-ops-coordinator"
  | "payroll-controller"
  | "finance-controller"
  | "ops-manager"
  | "procurement-lead"
  | "it-asset-admin"
  | "hospital-admissions-clerk"
  | "hospital-discharge-planner"
  | "hospital-bed-manager"
  | "hospital-compliance-officer"
  | "recruiting-coordinator"
  | "hiring-manager"
  | "talent-acquisition-lead";

export interface SloDeclaration {
  max_duration_seconds: number;
  min_success_rate: number;
  openslo_ref?: string;
}

export interface RuntimeExpectations {
  expected_duration_seconds_p50: number;
  expected_duration_seconds_p99: number;
}

export interface CostModel {
  estimated_usd_per_execution_p50: number;
  foundry_inference_usd: number;
  connector_call_usd: number;
  storage_usd: number;
}

export interface CatalogItem {
  template_id: string;
  name: string;
  description: string;
  persona: Persona;
  vertical: Vertical;
  tags: string[];
  compliance_flags: string[];
  slo: SloDeclaration;
  runtime_expectations: RuntimeExpectations;
  cost_model: CostModel;
  connector_count: number;
  node_count: number;
  definition_path: string;
  explainer_path: string;
  fixture_path: string;
  test_mode_supported: boolean;
  live_mode_supported: boolean;
}

export interface CatalogIndex {
  schema_version: string;
  generated_at: string;
  count: number;
  verticals: Vertical[];
  personas: Persona[];
  items: CatalogItem[];
}

export interface CatalogFilter {
  vertical?: Vertical | "all";
  persona?: Persona | "all";
  query?: string;
}

export function filterCatalog(index: CatalogIndex, filter: CatalogFilter): CatalogItem[] {
  const q = (filter.query ?? "").trim().toLowerCase();
  return index.items.filter((it) => {
    if (filter.vertical && filter.vertical !== "all" && it.vertical !== filter.vertical) return false;
    if (filter.persona && filter.persona !== "all" && it.persona !== filter.persona) return false;
    if (q) {
      const hay = `${it.name} ${it.description} ${it.tags.join(" ")} ${it.compliance_flags.join(" ")}`.toLowerCase();
      if (!hay.includes(q)) return false;
    }
    return true;
  });
}
