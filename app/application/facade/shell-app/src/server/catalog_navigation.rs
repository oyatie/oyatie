// ponytail: This bridge serves the oversized legacy shell view; use an anchor when that view is split.
pub(super) const SCRIPT: &str = r#"
function initializeOntologySelection() {
  const graph = document.querySelector('[data-ontology-console]');
  graph?.querySelector('[data-ontology-detail]')?.setAttribute('role', 'status');
  graph?.querySelectorAll('[data-ontology-node]').forEach((node) => {
    node.setAttribute('aria-pressed', String(node.classList.contains('selected')));
  });
}
initializeOntologySelection();
const host = document.getElementById('dashboard-island-root');
if (host) new MutationObserver(initializeOntologySelection).observe(host, { childList: true, subtree: true });
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
    const panel = document.getElementById('ontology-command-console');
    if (!panel) return;
    window.location.hash = panel.id;
    panel.scrollIntoView();
    panel.tabIndex = -1;
    panel.focus({ preventScroll: true });
    return;
  }

  const node = event.target.closest('[data-ontology-node]');
  const action = event.target.closest('[data-ontology-action]');
  const graph = (node || action)?.closest('[data-ontology-console]');
  const detail = graph?.querySelector('[data-ontology-detail]');
  if (!detail) return;
  detail.setAttribute('role', 'status');
  if (node) {
    selectOntologyNode(graph, node, detail);
  } else if (action?.dataset.ontologyAction === 'lineage') {
    const facts = [...graph.querySelectorAll('[data-ontology-fact]')].map((fact) =>
      [fact.querySelector('strong')?.textContent, fact.querySelector('em')?.textContent]
        .filter(Boolean).join(' · '));
    detail.textContent = facts.length
      ? `Current permitted relations: ${facts.join('; ')}`
      : 'No permitted relations in this view.';
  } else if (action?.dataset.ontologyAction === 'policy' || action?.dataset.ontologyAction === 'evidence') {
    const target = graph.querySelector(action.dataset.ontologyAction === 'policy'
      ? '[data-ontology-node="Policy"]' : '[data-ontology-node="Evidence"]');
    if (!target) return;
    selectOntologyNode(graph, target, detail);
    target.focus();
  } else if (action?.dataset.ontologyAction === 'inspect-fact') {
    const fact = action.closest('[data-ontology-fact]');
    if (!fact) return;
    detail.textContent = [fact.querySelector('strong'), fact.querySelector('em'), fact.querySelector('p')]
      .map((item) => item?.textContent).filter(Boolean).join(' · ');
  }
});
"#;
