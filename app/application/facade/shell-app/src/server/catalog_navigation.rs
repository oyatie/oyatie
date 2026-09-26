// ponytail: This bridge serves the oversized legacy shell view; use an anchor when that view is split.
pub(super) const SCRIPT: &str = r#"
document.addEventListener('click', (event) => {
  if (!(event.target instanceof Element)) return;
  const button = event.target.closest('[data-catalog-action="open"]');
  const card = button?.closest('[data-catalog-module]');
  if (!card?.querySelector('[data-sidepeek-id="CAT-OYATIE-ONTOLOGY"]')) return;
  const panel = document.getElementById('ontology-command-console');
  if (!panel) return;
  window.location.hash = panel.id;
  panel.scrollIntoView();
  panel.tabIndex = -1;
  panel.focus({ preventScroll: true });
});
"#;
