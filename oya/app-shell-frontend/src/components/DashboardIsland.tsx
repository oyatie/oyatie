// Archived SolidJS transition dashboard surface.
// ADR-0393 keeps Leptos/Rust-WASM canonical; this file remains read-only
// migration evidence for the operator-console layout and render-envelope contract.
//
// Reactive model maps directly from Leptos signals to SolidJS createSignal/createResource:
//   Leptos: let (active_context, set_active_context) = signal(DemoContext::TenantAdmin)
//   Solid:  const [activeContext, setActiveContext] = createSignal<DemoContext>("tenant-admin")
//
// The render-envelope is fetched from the canonical Leptos service via fetchRenderEnvelope()
// and displayed in three role-scoped context views.

import {
  createResource,
  createSignal,
  For,
  Match,
  Show,
  Switch,
  type Component,
} from "solid-js";
import { fetchRenderEnvelope } from "~/lib/api";
import {
  DEMO_CONTEXT_LABELS,
  DEMO_CONTEXTS,
  type DemoContext,
  type TenantRenderEnvelope,
  type MetricCard,
  type ModuleCard,
  type WorkItem,
  type ScheduleItem,
  type MessageItem,
  type ApprovalItem,
  type WorkflowNode,
  type IntelligenceSuggestion,
  type OntologyFact,
} from "~/lib/render-envelope";

// --- Sub-components -------------------------------------------------------

const MetricCardView: Component<{ card: MetricCard }> = (props) => (
  <article class="metric-card">
    <p class="screen-anchor">{props.card.label.toUpperCase()}</p>
    <strong class="metric-value">{props.card.value}</strong>
    <span class="metric-detail">{props.card.detail}</span>
  </article>
);

const ModuleCardView: Component<{ card: ModuleCard }> = (props) => (
  <article class="module-card">
    <div class="module-card-head">
      <strong>{props.card.name}</strong>
      <span class="badge">{props.card.group}</span>
    </div>
    <p>{props.card.description}</p>
    <button type="button" class="module-action">
      {props.card.action_label}
    </button>
  </article>
);

const WorkItemView: Component<{ item: WorkItem }> = (props) => (
  <li class="item-list-row">
    <span
      class={`priority priority--${props.item.priority.toLowerCase()}`}
      aria-label={`Priority: ${props.item.priority}`}
    >
      {props.item.priority}
    </span>
    <div>
      <strong>{props.item.title}</strong>
      <p>{props.item.detail}</p>
    </div>
  </li>
);

const ScheduleItemView: Component<{ item: ScheduleItem }> = (props) => (
  <li class="timeline-row">
    <time class="timeline-time">{props.item.time}</time>
    <div>
      <strong>{props.item.title}</strong>
      <p>{props.item.detail}</p>
    </div>
  </li>
);

const MessageItemView: Component<{ item: MessageItem }> = (props) => (
  <li class="message-row">
    <div class="message-meta">
      <strong>{props.item.from}</strong>
      <span class="badge">{props.item.channel}</span>
    </div>
    <p>{props.item.preview}</p>
  </li>
);

const ApprovalItemView: Component<{ item: ApprovalItem }> = (props) => (
  <li class="approval-row">
    <div>
      <strong>{props.item.title}</strong>
      <span class="approval-requester">{props.item.requester}</span>
    </div>
    <p class="approval-risk">{props.item.risk_note}</p>
    <div class="approval-actions">
      <button type="button">Review</button>
      <button type="button" class="primary">Approve</button>
    </div>
  </li>
);

const WorkflowNodeView: Component<{ node: WorkflowNode; isSelected: boolean; onSelect: () => void }> = (
  props,
) => (
  <button
    type="button"
    class={`workflow-node workflow-node--${props.node.kind.toLowerCase()} ${props.isSelected ? "is-selected" : ""}`}
    style={{ left: `${props.node.x}px`, top: `${props.node.y}px` }}
    onClick={props.onSelect}
    aria-pressed={props.isSelected}
    title={props.node.explanation}
  >
    <span class="workflow-node-kind">{props.node.kind}</span>
    <strong>{props.node.label}</strong>
  </button>
);

const IntelligenceSuggestionView: Component<{ suggestion: IntelligenceSuggestion }> = (props) => (
  <li class="suggestion-row">
    <div class="suggestion-head">
      <span class="ai-chip" aria-hidden="true">✦</span>
      <strong>{props.suggestion.title}</strong>
    </div>
    <p>{props.suggestion.body}</p>
    <small class="suggestion-guardrail">{props.suggestion.guardrail}</small>
  </li>
);

const OntologyFactView: Component<{ fact: OntologyFact }> = (props) => (
  <li class="ontology-row">
    <strong>{props.fact.entity}</strong>
    <span class="ontology-relation">{props.fact.relation}</span>
    <p>{props.fact.access_reason}</p>
  </li>
);

// --- Context switcher -------------------------------------------------------

const ContextSwitcher: Component<{
  active: DemoContext;
  onChange: (ctx: DemoContext) => void;
}> = (props) => (
  <section class="context-switcher" aria-label="Render-envelope context switcher">
    <div>
      <p class="screen-anchor">ROLE CONTEXT</p>
      <h2>Select render context</h2>
      <p class="context-switcher-note">
        Server-derived render envelopes shape each context. Healthcare modules are absent
        from non-accredited contexts server-side, not hidden client-side.
      </p>
    </div>
    <div class="context-grid" role="group" aria-label="Available render-envelope contexts">
      <For each={DEMO_CONTEXTS}>
        {(ctx) => (
          <button
            type="button"
            class={`context-card ${props.active === ctx ? "selected" : ""}`}
            aria-pressed={props.active === ctx}
            onClick={() => props.onChange(ctx)}
          >
            <span class="context-icon" aria-hidden="true">
              {ctx === "tenant-admin" ? "⚿" : ctx === "corporate-office" ? "▦" : "✚"}
            </span>
            <strong>{DEMO_CONTEXT_LABELS[ctx]}</strong>
          </button>
        )}
      </For>
    </div>
  </section>
);

// --- Workflow canvas (first slice — linear node display) --------------------

const WorkflowCanvas: Component<{
  nodes: WorkflowNode[];
  selectedNodeId: string;
  onSelectNode: (id: string) => void;
}> = (props) => (
  <div
    class="workflow-canvas"
    role="region"
    aria-label="Workflow node canvas"
    style={{ position: "relative", height: "10rem", overflow: "auto" }}
  >
    <For each={props.nodes}>
      {(node) => (
        <WorkflowNodeView
          node={node}
          isSelected={props.selectedNodeId === node.id}
          onSelect={() => props.onSelectNode(node.id)}
        />
      )}
    </For>
  </div>
);

// --- Main DashboardIsland --------------------------------------------------

const DashboardIsland: Component = () => {
  const [activeContext, setActiveContext] = createSignal<DemoContext>("tenant-admin");
  const [selectedNodeId, setSelectedNodeId] = createSignal<string>("");

  // SolidJS createResource: automatically refetches when activeContext() changes.
  // Mirrors Leptos spawn_local + set_envelope pattern.
  // The fetcher wrapper strips the ResourceFetcherInfo second arg so fetchRenderEnvelope
  // (which expects RequestInit | undefined) gets a compatible signature.
  const [envelope] = createResource<TenantRenderEnvelope, DemoContext>(
    activeContext,
    (ctx) => fetchRenderEnvelope(ctx),
  );

  // Set initial selected node when envelope resolves.
  const firstNodeId = () => envelope()?.workflow.nodes[0]?.id ?? "";

  return (
    <div class="dashboard-island" aria-label="Operator console dashboard">

      {/* Context switcher — ported from Leptos DashboardIsland context buttons */}
      <ContextSwitcher active={activeContext()} onChange={(ctx) => {
        setActiveContext(ctx);
        setSelectedNodeId("");
      }} />

      {/* Loading / error states */}
      <Switch>
        <Match when={envelope.loading}>
          <div class="island-loading" role="status" aria-live="polite">
            <span class="screen-anchor">LOADING</span>
            <p>Fetching render envelope…</p>
          </div>
        </Match>
        <Match when={envelope.error}>
          <div class="island-error" role="alert">
            <p class="screen-anchor">ENVELOPE ERROR</p>
            <strong>Could not load render envelope</strong>
            <p>
              {envelope.error instanceof Error
                ? envelope.error.message
                : "Unknown error — check that the canonical Leptos render-envelope service is running on port 3000."}
            </p>
            <p class="shell-notice">
              This archived transition shell stays non-mutating when the Leptos service is unavailable. Metrics, workflow
              canvas, and module cards populate only after the render-envelope backend is reachable.
            </p>
          </div>
        </Match>
      </Switch>

      <Show when={envelope()}>
        {(env) => (
          <>
            {/* Envelope banner — server derivation note */}
            <section class="envelope-banner" aria-label="Render envelope context">
              <div>
                <p class="screen-anchor">SERVER-DERIVED ENVELOPE</p>
                <strong>{env().tenant_name}</strong>
                <p>{env().role_name}</p>
                <p class="envelope-detail">{env().server_derivation_note}</p>
                <p class="shell-notice">{env().accreditation.label}</p>
              </div>
              <div class="envelope-activity">
                <p class="screen-anchor">PRODUCT ACTIVITY</p>
                <strong>{env().product_activity.active_context}</strong>
                <small>{env().product_activity.status_label}</small>
                <ol class="activity-spine" aria-label="Product activity spine">
                  <For each={env().product_activity.steps}>
                    {(step) => (
                      <li
                        class={`spine-step spine-step--${step.state}`}
                        title={step.detail}
                      >
                        <a href={step.target} class="spine-label">
                          {step.label}
                        </a>
                        <span class="spine-surface">{step.surface}</span>
                      </li>
                    )}
                  </For>
                </ol>
              </div>
            </section>

            {/* Metric grid */}
            <section aria-labelledby="metrics-title">
              <h2 id="metrics-title" class="panel-heading">
                <span class="screen-anchor">KEY METRICS</span>
                This week
              </h2>
              <div class="metric-grid">
                <For each={env().metrics}>
                  {(card) => <MetricCardView card={card} />}
                </For>
              </div>
            </section>

            {/* Main dashboard grid: tasks + schedule + messages + approvals */}
            <div class="dashboard-grid">
              {/* Daily tasks */}
              <section
                id="tasks-title"
                class="panel col-span-4"
                aria-labelledby="tasks-heading"
              >
                <div class="panel-head">
                  <h3 id="tasks-heading">
                    <span class="screen-anchor">DAILY TASKS</span>
                    Tasks
                  </h3>
                </div>
                <ol class="item-list" aria-label="Daily tasks">
                  <For each={env().daily_tasks}>
                    {(item) => <WorkItemView item={item} />}
                  </For>
                </ol>
              </section>

              {/* Schedule */}
              <section
                id="schedule-title"
                class="panel col-span-4"
                aria-labelledby="schedule-heading"
              >
                <div class="panel-head">
                  <h3 id="schedule-heading">
                    <span class="screen-anchor">TODAY'S SCHEDULE</span>
                    Schedule
                  </h3>
                </div>
                <ol class="timeline-list" aria-label="Today's schedule">
                  <For each={env().schedule}>
                    {(item) => <ScheduleItemView item={item} />}
                  </For>
                </ol>
              </section>

              {/* Messages */}
              <section
                id="work-hub"
                class="panel col-span-4"
                aria-labelledby="messages-heading"
              >
                <div class="panel-head">
                  <h3 id="messages-heading">
                    <span class="screen-anchor">MESSAGES</span>
                    Messenger · Mail
                  </h3>
                </div>
                <ol class="message-list" aria-label="Recent messages">
                  <For each={env().messages}>
                    {(item) => <MessageItemView item={item} />}
                  </For>
                </ol>
              </section>
            </div>

            {/* Approvals */}
            <section class="panel" aria-labelledby="approvals-heading">
              <div class="panel-head">
                <h3 id="approvals-heading">
                  <span class="screen-anchor">PENDING APPROVALS</span>
                  Approvals
                </h3>
                <span class="badge warning">{env().approvals.length} pending</span>
              </div>
              <ol class="approval-list" aria-label="Pending approvals">
                <For each={env().approvals}>
                  {(item) => <ApprovalItemView item={item} />}
                </For>
              </ol>
            </section>

            {/* Workflow Studio */}
            <section
              id="workflow-studio"
              class="panel studio-panel"
              aria-labelledby="workflow-heading"
            >
              <div class="panel-head">
                <div>
                  <p class="screen-anchor">WORKFLOW STUDIO</p>
                  <h3 id="workflow-heading">{env().workflow.name}</h3>
                  <p class="panel-intro">{env().workflow.goal}</p>
                </div>
              </div>
              <WorkflowCanvas
                nodes={env().workflow.nodes}
                selectedNodeId={selectedNodeId() || firstNodeId()}
                onSelectNode={setSelectedNodeId}
              />
              <Show when={(selectedNodeId() || firstNodeId())}>
                {(nodeId) => {
                  const node = () =>
                    env().workflow.nodes.find((n: WorkflowNode) => n.id === nodeId());
                  return (
                    <Show when={node()}>
                      {(n) => (
                        <aside class="workflow-node-inspector" aria-label="Selected node inspector">
                          <p class="screen-anchor">NODE INSPECTOR</p>
                          <strong>{n().label}</strong>
                          <span class="badge">{n().kind}</span>
                          <p>{n().explanation}</p>
                        </aside>
                      )}
                    </Show>
                  );
                }}
              </Show>
            </section>

            {/* Module catalog */}
            <section id="modules-title" aria-labelledby="modules-heading">
              <h3 id="modules-heading" class="panel-heading">
                <span class="screen-anchor">SERVICE CATALOG</span>
                Modules
              </h3>
              <div class="module-grid">
                <For each={env().modules}>
                  {(card) => <ModuleCardView card={card} />}
                </For>
              </div>
            </section>

            {/* Intelligence / copilot rail */}
            <section
              id="intelligence-title"
              class="panel"
              aria-labelledby="intelligence-heading"
            >
              <div class="panel-head">
                <h3 id="intelligence-heading">
                  <span class="screen-anchor">COPILOT SUGGESTIONS</span>
                  <span aria-hidden="true">✦ </span>Intelligence
                </h3>
              </div>
              <ol class="suggestion-list" aria-label="Intelligence suggestions">
                <For each={env().intelligence}>
                  {(s) => <IntelligenceSuggestionView suggestion={s} />}
                </For>
              </ol>
            </section>

            {/* Ontology / object graph */}
            <section
              id="ontology-title"
              class="panel"
              aria-labelledby="ontology-heading"
            >
              <div class="panel-head">
                <h3 id="ontology-heading">
                  <span class="screen-anchor">OBJECT GRAPH</span>
                  Ontology
                </h3>
              </div>
              <ol class="ontology-list" aria-label="Object graph facts">
                <For each={env().ontology}>
                  {(fact) => <OntologyFactView fact={fact} />}
                </For>
              </ol>
            </section>

            {/* Omitted capability note */}
            <p class="omitted-note" role="note">
              {env().omitted_capability_note}
            </p>
          </>
        )}
      </Show>
    </div>
  );
};

export default DashboardIsland;
