<script lang="ts">
  /**
   * CatalogBrowser — the "Browse templates" surface in Workflow Studio.
   *
   * Renders all 25 templates with vertical + persona + search filters;
   * clicking a tile opens TemplateDetail; "Deploy in test mode" emits a
   * `deploy` event the parent route handler routes into the canvas.
   */
  import { onMount } from "svelte";
  import TemplateCard from "./TemplateCard.svelte";
  import TemplateDetail from "./TemplateDetail.svelte";
  import { filterCatalog, type CatalogIndex, type CatalogItem, type Vertical, type Persona } from "./types";
  import { loadCatalog, type FetchLike } from "./loader";

  export let fetcher: FetchLike;
  export let preloaded: CatalogIndex | null = null;

  let index: CatalogIndex | null = preloaded;
  let loadError: string | null = null;
  let selected: CatalogItem | null = null;
  let verticalFilter: Vertical | "all" = "all";
  let personaFilter: Persona | "all" = "all";
  let query = "";

  onMount(async () => {
    if (index) return;
    try {
      index = await loadCatalog(fetcher);
    } catch (err) {
      loadError = (err as Error).message;
    }
  });

  $: visible = index
    ? filterCatalog(index, { vertical: verticalFilter, persona: personaFilter, query })
    : [];

  function handleSelect(event: CustomEvent<CatalogItem>) {
    selected = event.detail;
  }

  function handleDeployTest(event: CustomEvent<CatalogItem>) {
    deployment = { item: event.detail, mode: "test" };
  }

  function handleDeployLive(event: CustomEvent<CatalogItem>) {
    deployment = { item: event.detail, mode: "live" };
  }

  // Surfaced so the host route can observe deployments via bind:deployment.
  export let deployment: { item: CatalogItem; mode: "test" | "live" } | null = null;
</script>

<div class="ws-catalog">
  <header class="ws-catalog-header">
    <h1>Browse templates</h1>
    <p>Pre-built workflows you can deploy in test mode in 3 parameters and 1 click.</p>
  </header>

  {#if loadError}
    <p class="ws-error" data-testid="catalog-error">{loadError}</p>
  {:else if !index}
    <p data-testid="catalog-loading">Loading template catalog...</p>
  {:else}
    <div class="ws-filters">
      <label>
        Vertical
        <select bind:value={verticalFilter} data-testid="filter-vertical">
          <option value="all">All</option>
          {#each index.verticals as v}
            <option value={v}>{v}</option>
          {/each}
        </select>
      </label>
      <label>
        Persona
        <select bind:value={personaFilter} data-testid="filter-persona">
          <option value="all">All</option>
          {#each index.personas as p}
            <option value={p}>{p}</option>
          {/each}
        </select>
      </label>
      <label class="ws-search">
        Search
        <input type="search" bind:value={query} placeholder="onboarding, payroll, hipaa..." data-testid="filter-query" />
      </label>
      <span class="ws-result-count" data-testid="result-count">{visible.length} of {index.count}</span>
    </div>

    <div class="ws-grid" data-testid="catalog-grid">
      {#each visible as item (item.template_id)}
        <TemplateCard {item} on:select={handleSelect} />
      {/each}
    </div>

    {#if selected}
      <div class="ws-detail-pane" data-testid="detail-pane">
        <TemplateDetail
          item={selected}
          on:close={() => (selected = null)}
          on:deploy-test={handleDeployTest}
          on:deploy-live={handleDeployLive}
        />
      </div>
    {/if}
  {/if}
</div>

<style>
  .ws-catalog {
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .ws-catalog-header h1 {
    margin: 0;
    font-size: 22px;
  }
  .ws-catalog-header p {
    margin: 4px 0 0;
    color: var(--ws-muted, #57606a);
  }
  .ws-filters {
    display: flex;
    align-items: end;
    gap: 12px;
    flex-wrap: wrap;
  }
  .ws-filters label {
    display: flex;
    flex-direction: column;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--ws-muted, #57606a);
    gap: 4px;
  }
  .ws-filters select,
  .ws-filters input {
    font-size: 13px;
    padding: 6px 10px;
    border: 1px solid var(--ws-border, #d0d7de);
    border-radius: 8px;
    background: #ffffff;
  }
  .ws-search input {
    min-width: 220px;
  }
  .ws-result-count {
    margin-left: auto;
    font-size: 12px;
    color: var(--ws-muted, #57606a);
  }
  .ws-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 12px;
  }
  .ws-detail-pane {
    margin-top: 8px;
  }
  .ws-error {
    color: #cf222e;
  }
</style>
