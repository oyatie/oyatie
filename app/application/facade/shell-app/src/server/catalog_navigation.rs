// ponytail: This bridge serves the oversized legacy shell view; use an anchor when that view is split.
pub(super) const SCRIPT: &str = r#"
let pendingOntologyFocus = null;
let pendingOntologyAction = null;
function ontologyIsCurrentDestination() {
  try {
    return decodeURIComponent(window.location.hash.slice(1)) === 'ontology-command-console';
  } catch {
    return false;
  }
}
function ontologyFacts(graph) {
  return JSON.stringify([...graph.querySelectorAll('[data-ontology-fact]')].map((fact) =>
    [fact.querySelector('strong'), fact.querySelector('em'), fact.querySelector('p')]
      .map((item) => item?.textContent)));
}
function factDetail(fact) {
  return [fact.querySelector('strong'), fact.querySelector('em'), fact.querySelector('p')]
    .map((item) => item?.textContent).filter(Boolean).join(' · ');
}
function lineageDetail(graph) {
  const facts = [...graph.querySelectorAll('[data-ontology-fact]')].map((fact) =>
    [fact.querySelector('strong')?.textContent, fact.querySelector('em')?.textContent]
      .filter(Boolean).join(' · '));
  return facts.length
    ? `Current permitted relations: ${facts.join('; ')}`
    : 'No permitted relations in this view.';
}
function focusPanel(id) {
  const panel = document.getElementById(id);
  if (!panel) return null;
  window.location.hash = `#${panel.id}`;
  panel.scrollIntoView();
  panel.tabIndex = -1;
  panel.focus({ preventScroll: true });
  return panel;
}
function initializeOntologySelection() {
  const graph = document.querySelector('[data-ontology-console]');
  graph?.querySelector('[data-ontology-detail]')?.setAttribute('role', 'status');
  graph?.querySelectorAll('[data-ontology-node]').forEach((node) => {
    node.setAttribute('aria-pressed', String(node.classList.contains('selected')));
  });
  if (pendingOntologyAction && !pendingOntologyAction.graph.isConnected && graph) {
    const previous = pendingOntologyAction;
    pendingOntologyAction = null;
    if (previous.context && graph.dataset.ontologyContext === previous.context &&
        ontologyFacts(graph) === previous.facts) {
      const detail = graph.querySelector('[data-ontology-detail]');
      if (!previous.node && previous.selection && detail) {
        const selected = [...graph.querySelectorAll('[data-ontology-node]')]
          .find((node) => node.dataset.ontologyNode === previous.selection);
        if (selected) selectOntologyNode(graph, selected, detail);
      }
      if (previous.node) {
        const match = [...graph.querySelectorAll('[data-ontology-node]')]
          .find((node) => node.dataset.ontologyNode === previous.node);
        if (match && detail) selectOntologyNode(graph, match, detail);
      } else if (previous.fact !== undefined) {
        const match = graph.querySelectorAll('[data-ontology-fact]')[previous.fact];
        if (match && detail) detail.textContent = factDetail(match);
      } else if (previous.lineage && detail) {
        detail.textContent = lineageDetail(graph);
      }
    }
  }
  if (pendingOntologyFocus && !pendingOntologyFocus.panel.isConnected && graph) {
    const previous = pendingOntologyFocus;
    const replacement = document.getElementById('ontology-command-console');
    if (replacement) {
      const active = document.activeElement;
      const restore = ontologyIsCurrentDestination() &&
        (!active || active === document.body || !active.isConnected) &&
        previous.context && graph.dataset.ontologyContext === previous.context &&
        ontologyFacts(graph) === previous.facts;
      pendingOntologyFocus = null;
      if (restore) {
        let target = replacement;
        if (previous.target.dataset.ontologyNode) {
          target = [...graph.querySelectorAll('[data-ontology-node]')]
            .find((node) => node.dataset.ontologyNode === previous.target.dataset.ontologyNode);
        } else if (previous.target.dataset.ontologyAction) {
          const fact = previous.target.closest('[data-ontology-fact]');
          const scope = fact ? graph.querySelectorAll('[data-ontology-fact]')[
            [...previous.panel.querySelectorAll('[data-ontology-fact]')].indexOf(fact)] : graph;
          target = [...(scope?.querySelectorAll('[data-ontology-action]') || [])]
            .find((action) => action.dataset.ontologyAction === previous.target.dataset.ontologyAction);
        }
        target?.scrollIntoView();
        if (target === replacement) target.tabIndex = -1;
        target?.focus({ preventScroll: true });
      }
    }
  }
}
const host = document.getElementById('dashboard-island-root');
if (host) new MutationObserver(initializeOntologySelection).observe(host, { childList: true, subtree: true });
document.addEventListener('focusin', (event) => {
  const panel = document.getElementById('ontology-command-console');
  const graph = panel?.querySelector('[data-ontology-console]');
  if (graph && panel.contains(event.target)) {
    pendingOntologyFocus = {
      panel, target: event.target, facts: ontologyFacts(graph), context: graph.dataset.ontologyContext,
    };
    return;
  }
  if (pendingOntologyFocus && event.target !== document.body &&
      !pendingOntologyFocus.panel.contains(event.target)) pendingOntologyFocus = null;
});
if (ontologyIsCurrentDestination() && document.activeElement === document.body) {
  const panel = document.getElementById('ontology-command-console');
  if (panel) {
    panel.tabIndex = -1;
    panel.scrollIntoView();
    panel.focus({ preventScroll: true });
  }
}
initializeOntologySelection();
function selectOntologyNode(graph, node, detail) {
  for (const candidate of graph.querySelectorAll('[data-ontology-node]')) {
    const selected = candidate === node;
    candidate.classList.toggle('selected', selected);
    candidate.setAttribute('aria-pressed', String(selected));
  }
  detail.textContent = `${node.dataset.ontologyNode} selected · ${node.dataset.sidepeekDesc}`;
}
document.addEventListener('click', (event) => {
  if (!(event.target instanceof Element)) return;
  const button = event.target.closest('[data-catalog-action="open"]');
  const card = button?.closest('[data-catalog-module]');
  if (card?.querySelector('[data-sidepeek-id="CAT-OYATIE-ONTOLOGY"]')) {
    focusPanel('ontology-command-console');
    return;
  }
  if (pendingOntologyFocus && !pendingOntologyFocus.panel.contains(event.target)) pendingOntologyFocus = null;
  pendingOntologyAction = null;

  const node = event.target.closest('[data-ontology-node]');
  const action = event.target.closest('[data-ontology-action]');
  const graph = (node || action)?.closest('[data-ontology-console]');
  const detail = graph?.querySelector('[data-ontology-detail]');
  if (!detail) return;
  detail.setAttribute('role', 'status');
  if (node) {
    selectOntologyNode(graph, node, detail);
    pendingOntologyAction = { graph, facts: ontologyFacts(graph), node: node.dataset.ontologyNode };
  } else if (action?.dataset.ontologyAction === 'lineage') {
    detail.textContent = lineageDetail(graph);
    pendingOntologyAction = { graph, facts: ontologyFacts(graph), lineage: true };
  } else if (action?.dataset.ontologyAction === 'policy' || action?.dataset.ontologyAction === 'evidence') {
    const target = graph.querySelector(action.dataset.ontologyAction === 'policy'
      ? '[data-ontology-node="Policy"]' : '[data-ontology-node="Evidence"]');
    if (!target) return;
    selectOntologyNode(graph, target, detail);
    pendingOntologyAction = { graph, facts: ontologyFacts(graph), node: target.dataset.ontologyNode };
    target.focus();
  } else if (action?.dataset.ontologyAction === 'inspect-fact') {
    const fact = action.closest('[data-ontology-fact]');
    if (!fact) return;
    detail.textContent = factDetail(fact);
    pendingOntologyAction = {
      graph, facts: ontologyFacts(graph),
      fact: [...graph.querySelectorAll('[data-ontology-fact]')].indexOf(fact),
    };
  } else if (action?.dataset.ontologyAction === 'route-workflow') {
    focusPanel('workflow-studio');
  }
  if (pendingOntologyAction) {
    pendingOntologyAction.context = graph.dataset.ontologyContext;
    pendingOntologyAction.selection = [...graph.querySelectorAll('[data-ontology-node]')]
      .find((candidate) => candidate.classList.contains('selected'))?.dataset.ontologyNode;
  }
});
"#;

#[cfg(test)]
#[path = "catalog_navigation_test.rs"]
mod tests;
