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
  const inspect = event.target.closest('[data-ontology-action="inspect-fact"]');
  const graph = (node || inspect)?.closest('[data-ontology-console]');
  const detail = graph?.querySelector('[data-ontology-detail]');
  if (!detail) return;
  detail.setAttribute('role', 'status');
  if (node) {
    for (const candidate of graph.querySelectorAll('[data-ontology-node]')) {
      const selected = candidate === node;
      candidate.classList.toggle('selected', selected);
      candidate.setAttribute('aria-pressed', String(selected));
    }
    detail.textContent = `${node.dataset.ontologyNode} selected · ${node.dataset.sidepeekDesc}`;
  } else {
    const fact = inspect.closest('[data-ontology-fact]');
    if (!fact) return;
    detail.textContent = [fact.querySelector('strong'), fact.querySelector('em'), fact.querySelector('p')]
      .map((item) => item?.textContent).filter(Boolean).join(' · ');
  }
});
"#;
