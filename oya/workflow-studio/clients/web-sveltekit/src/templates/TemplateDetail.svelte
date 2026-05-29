<script lang="ts">
  /**
   * TemplateDetail — full detail view for a single selected template.
   * Pure presentational. The catalog page is responsible for fetching the
   * full WorkflowStudioTemplate JSON when the user opens detail; this
   * component just renders the projection it is given.
   */
  import { createEventDispatcher } from "svelte";
  import type { CatalogItem } from "./types";

  export let item: CatalogItem;

  const dispatch = createEventDispatcher<{
    "deploy-test": CatalogItem;
    "deploy-live": CatalogItem;
    close: void;
  }>();
</script>

<section class="ws-template-detail" data-template-id={item.template_id}>
  <header>
    <div>
      <h2>{item.name}</h2>
      <p class="ws-template-id">{item.template_id}</p>
    </div>
    <button type="button" class="ws-close" on:click={() => dispatch("close")}>x</button>
  </header>

  <p>{item.description}</p>

  <dl class="ws-meta-grid">
    <div>
      <dt>Persona</dt>
      <dd>{item.persona}</dd>
    </div>
    <div>
      <dt>Vertical</dt>
      <dd>{item.vertical}</dd>
    </div>
    <div>
      <dt>SLO max duration</dt>
      <dd>{item.slo.max_duration_seconds}s</dd>
    </div>
    <div>
      <dt>SLO success rate</dt>
      <dd>{(item.slo.min_success_rate * 100).toFixed(2)}%</dd>
    </div>
    <div>
      <dt>p50 runtime</dt>
      <dd>{item.runtime_expectations.expected_duration_seconds_p50}s</dd>
    </div>
    <div>
      <dt>p99 runtime</dt>
      <dd>{item.runtime_expectations.expected_duration_seconds_p99}s</dd>
    </div>
    <div>
      <dt>Estimated cost / run</dt>
      <dd>${item.cost_model.estimated_usd_per_execution_p50}</dd>
    </div>
    <div>
      <dt>Connectors</dt>
      <dd>{item.connector_count}</dd>
    </div>
    <div>
      <dt>Nodes</dt>
      <dd>{item.node_count}</dd>
    </div>
  </dl>

  <h3>Compliance</h3>
  <ul class="ws-compliance-list">
    {#each item.compliance_flags as flag}
      <li>{flag}</li>
    {/each}
  </ul>

  <h3>Tags</h3>
  <ul class="ws-tag-list">
    {#each item.tags as tag}
      <li>{tag}</li>
    {/each}
  </ul>

  <footer class="ws-actions">
    <button
      type="button"
      class="ws-btn ws-btn-test"
      disabled={!item.test_mode_supported}
      on:click={() => dispatch("deploy-test", item)}
      data-action="deploy-test"
    >
      Deploy in test mode (mocked connectors)
    </button>
    <button
      type="button"
      class="ws-btn ws-btn-live"
      disabled={!item.live_mode_supported}
      on:click={() => dispatch("deploy-live", item)}
      data-action="deploy-live"
    >
      Switch to live mode
    </button>
  </footer>
</section>

<style>
  .ws-template-detail {
    border: 1px solid var(--ws-border, #d0d7de);
    border-radius: 12px;
    padding: 20px;
    background: var(--ws-surface, #ffffff);
  }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }
  h2 {
    margin: 0 0 4px;
    font-size: 18px;
  }
  .ws-template-id {
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-size: 11px;
    color: var(--ws-muted, #57606a);
    margin: 0;
  }
  .ws-close {
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 14px;
  }
  .ws-meta-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
    margin: 16px 0;
  }
  .ws-meta-grid dt {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--ws-muted, #57606a);
  }
  .ws-meta-grid dd {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
  }
  .ws-compliance-list,
  .ws-tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 0;
    list-style: none;
    margin: 4px 0 16px;
  }
  .ws-compliance-list li,
  .ws-tag-list li {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--ws-pill-bg, #eaeef2);
    color: var(--ws-pill-fg, #57606a);
  }
  .ws-actions {
    display: flex;
    gap: 8px;
    margin-top: 16px;
  }
  .ws-btn {
    flex: 1;
    padding: 10px 14px;
    border-radius: 8px;
    border: 1px solid var(--ws-border, #d0d7de);
    background: var(--ws-surface, #ffffff);
    cursor: pointer;
    font-weight: 600;
  }
  .ws-btn[disabled] {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .ws-btn-test {
    border-color: var(--ws-accent, #0969da);
    color: var(--ws-accent, #0969da);
  }
  .ws-btn-live {
    background: var(--ws-accent, #0969da);
    color: #ffffff;
    border-color: var(--ws-accent, #0969da);
  }
</style>
