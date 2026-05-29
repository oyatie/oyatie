<script lang="ts">
  /**
   * TemplateCard — single-template tile for the Workflow Studio catalog browser.
   *
   * Pure presentational component: takes a CatalogItem and emits a `select`
   * event when the user clicks. No network, no stores, no side effects —
   * keeps the SSR + vitest renderer paths trivial.
   */
  import { createEventDispatcher } from "svelte";
  import type { CatalogItem } from "./types";

  export let item: CatalogItem;

  const dispatch = createEventDispatcher<{ select: CatalogItem }>();
</script>

<button
  type="button"
  class="ws-template-card"
  data-template-id={item.template_id}
  on:click={() => dispatch("select", item)}
>
  <header>
    <h3>{item.name}</h3>
    <span class="ws-vertical-pill" data-vertical={item.vertical}>{item.vertical}</span>
  </header>

  <p class="ws-description">{item.description}</p>

  <dl class="ws-meta">
    <div>
      <dt>Persona</dt>
      <dd>{item.persona}</dd>
    </div>
    <div>
      <dt>Connectors</dt>
      <dd>{item.connector_count}</dd>
    </div>
    <div>
      <dt>Nodes</dt>
      <dd>{item.node_count}</dd>
    </div>
    <div>
      <dt>SLO max</dt>
      <dd>{item.slo.max_duration_seconds}s</dd>
    </div>
    <div>
      <dt>Cost p50</dt>
      <dd>${item.cost_model.estimated_usd_per_execution_p50}</dd>
    </div>
  </dl>

  <ul class="ws-compliance">
    {#each item.compliance_flags as flag}
      <li data-flag={flag}>{flag}</li>
    {/each}
  </ul>
</button>

<style>
  .ws-template-card {
    display: block;
    width: 100%;
    text-align: left;
    border: 1px solid var(--ws-border, #d0d7de);
    border-radius: 12px;
    padding: 16px;
    background: var(--ws-surface, #ffffff);
    cursor: pointer;
    transition: border-color 120ms ease, box-shadow 120ms ease;
  }
  .ws-template-card:hover {
    border-color: var(--ws-accent, #0969da);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 8px;
  }
  h3 {
    font-size: 15px;
    margin: 0;
  }
  .ws-vertical-pill {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--ws-pill-bg, #eaeef2);
    color: var(--ws-pill-fg, #57606a);
  }
  .ws-description {
    font-size: 13px;
    color: var(--ws-muted, #57606a);
    margin: 0 0 12px;
    line-height: 1.4;
  }
  .ws-meta {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 8px;
    margin: 0 0 12px;
  }
  .ws-meta div {
    display: flex;
    flex-direction: column;
  }
  .ws-meta dt {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--ws-muted, #57606a);
  }
  .ws-meta dd {
    margin: 0;
    font-size: 12px;
    font-weight: 600;
  }
  .ws-compliance {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 0;
    margin: 0;
    list-style: none;
  }
  .ws-compliance li {
    font-size: 10px;
    padding: 2px 6px;
    background: var(--ws-tag-bg, #f6f8fa);
    border-radius: 4px;
    color: var(--ws-muted, #57606a);
  }
</style>
