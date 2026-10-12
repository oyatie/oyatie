pub(super) const PRELUDE: &str = r##"
(() => {
  let current = null;
  let state = null;
  let origins = {};
  let forwarding = false;
  let ownHash = null;
  const host = document.getElementById('dashboard-island-root');
  const all = (root, selector) => [...(root?.querySelectorAll(selector) || [])];
  const unique = (root, selector) => { const nodes = all(root, selector); return nodes.length === 1 ? nodes[0] : null; };
  const byId = id => { const nodes = all(document, '[id]').filter(node => node.id === id); return id && nodes.length === 1 ? nodes[0] : null; };
  const owner = () => { const root = unique(document, '#finance-preview'); return unique(document, '#dashboard-island-root') === host && host?.contains(root) ? root : null; };
  const panelSelector = '#finance-panel-leave, #leave-time, [data-leave-source][data-leave-context]';
  const panel = () => unique(owner(), panelSelector);
  const localControls = '#leave-sample-search, #leave-sample-backup, [data-finance-action], [data-finance-anchor-action], [data-leave-close]';
  const bindings = new WeakMap(), boundAnchors = new WeakSet();
  const inboundSelector = '[data-finance-anchor-action="route-leave"], [data-finance-command-action="open-leave"], [data-finance-command-action="route-leave"], [data-workforce-anchor-action="route-leave"]';
  const isEntry = control => control.matches('[data-finance-tab="leave"], [data-finance-side="leave"]') || control.matches('a.rail-nav') && ['#finance-panel-leave','#leave-time'].includes(control.getAttribute('href'));
  const isOwned = control => control.closest(panelSelector) || isEntry(control) || control.matches(inboundSelector);
  let available = false;
  function facts(p) {
    if (!p) return null;
    const copy = p.cloneNode(true);
    all(copy, '[data-leave-projected]').forEach(node => { node.textContent = ''; });
    const projected = new Set(['aria-disabled','tabindex','hidden','open','data-leave-mode']);
    return JSON.stringify([copy.textContent, [copy, ...all(copy, '*')].map(node => [node.tagName, [...node.attributes]
      .filter(attr => !projected.has(attr.name) && !(attr.name === 'disabled' && node.matches(localControls)))
      .map(attr => [attr.name, attr.name === 'class' ? attr.value.split(/\s+/).filter(token => token !== 'active').join(' ') : attr.value])
      .filter(([name, value]) => name !== 'class' || value !== '').sort(([a], [b]) => a.localeCompare(b))])]);
  }
  const key = p => JSON.stringify([p.closest('#finance-preview')?.dataset.financeSource, facts(p)]);
  function valid(p) {
    try {
      const root = owner(), source = JSON.parse(root?.dataset.financeSource), expected = document.createElement('div');
      if (!p || !root?.contains(p) || !Array.isArray(source) || source.length !== 5 || typeof source[4] !== 'string' || !source[4]
        || !['tenant-admin','corporate-office','healthcare-clinician'].includes(source[0])
        || source[0] !== root.dataset.financeContext || source[0] !== p.dataset.leaveContext
        || typeof source[1] !== 'string' || !source[1] || typeof source[3] !== 'string' || !source[3]
        || source[1] !== root.dataset.financeLabel || source[3] !== root.dataset.financeHubSource
        || !source[2] || typeof source[2] !== 'object' || Array.isArray(source[2]) || Object.keys(source[2]).length !== 3
        || ['commercial_examples','finops_examples','filing_examples'].some(name => source[2][name] !== (source[0] !== 'healthcare-clinician'))
        || JSON.stringify(source[2]) !== root.dataset.financeScope
        || !['ssr','island'].includes(root.dataset.financeMode) || p.dataset.leaveMode !== root.dataset.financeMode
        || !['true','false'].includes(root.dataset.financeNative) || bindings.has(p) && bindings.get(p) !== key(p)) return false;
      let body = source[4];
      if (root.dataset.financeNative === 'false') for (const [native, compat] of [['finance-panel-ledger','ledger-preview'],['finance-panel-vendors','vendors-spend'],['finance-panel-billing','billing-tax'],['finance-panel-leave','leave-time']]) for (const attribute of ['id','aria-controls','href']) { const prefix = attribute === 'href' ? '#' : ''; body = body.replaceAll(`${attribute}="${prefix}${native}"`, `${attribute}="${prefix}${compat}"`); }
      expected.innerHTML = body;
      if (facts(p) !== facts(unique(expected, '[data-leave-source][data-leave-context]')) || all(p, '[id]').some(node => byId(node.id) !== node)) return false;
      const tab = unique(root, '#finance-tab-leave'), tabs = tab?.closest('.finance-tabs[role="tablist"]');
      if (!tabs || tab.getAttribute('aria-controls') !== p.id || p.getAttribute('aria-labelledby') !== tab.id
        || byId(p.id) !== p || unique(document, '#finance-tab-leave') !== tab) return false;
      bindings.set(p, key(p));
      const dashboard = root.closest('[data-island="render-envelope-dashboard"]'), picker = unique(dashboard, '[data-sample-context-requested]');
      return root.dataset.financeSourceAvailable === 'true' && root.dataset.financeAvailable === 'true'
        && dashboard?.getAttribute('aria-busy') === 'false' && !dashboard.querySelector('.fetch-error')
        && dashboard.dataset.fragmentNavigation === root.dataset.financeMode
        && picker?.dataset.sampleContextRequested === source[0] && picker.dataset.sampleContextDisplayed === source[0];
    } catch { return false; }
  }
  function normalize(p, ready) {
    all(p, localControls).forEach(node => { if (node.disabled !== !ready) node.disabled = !ready; });
    const tab = p?.closest('#finance-preview')?.querySelector('#finance-tab-leave');
    if (tab && tab.disabled !== !ready) tab.disabled = !ready;
    if (!ready) { const node = p?.querySelector('[data-leave-status]'), text = 'Leave sample source pending, unavailable, invalid or ambiguous; controls disabled. No live HR effect.'; if (node && node.textContent !== text) node.textContent = text; }
  }
  function destinationReady(target, route) {
    const root = owner(), dashboard = root?.closest('[data-island="render-envelope-dashboard"]');
    if (!target?.isConnected || byId(target.id) !== target || !host.contains(target)
      || target.closest('[data-island="render-envelope-dashboard"]') !== dashboard || target.closest('[hidden], [inert], [aria-hidden="true"]')) return false;
    if (route === 'finance-panel-ledger') return root.contains(target);
    if (route === 'workflow-studio') return target.dataset.workflowContext === root.dataset.financeContext && target.dataset.workflowMode === root.dataset.financeMode;
    if (route === 'cockpit-panel-policy') return target.dataset.policyContext === root.dataset.financeContext && target.dataset.policyMode === root.dataset.financeMode;
    if (route === 'cloud-ops-cockpit') { const topology = unique(target, '[data-cloud-root="topology"]'); return topology?.dataset.cloudContext === root.dataset.financeContext && topology.dataset.cloudMode === root.dataset.financeMode && topology.dataset.cloudLabel === root.dataset.financeLabel && topology.dataset.cloudHubSource === root.dataset.financeHubSource; }
    if (['resource-panel-audit','evidence-spine'].includes(route)) {
      const evidence = unique(document, '#evidence-spine'), audit = unique(document, '#resource-audit-console');
      if (!evidence || !audit) return false;
      return [evidence, audit].every(node => host.contains(node) && node.dataset.aeContext === root.dataset.financeContext
        && node.dataset.aeMode === root.dataset.financeMode && node.dataset.aeHubSource === root.dataset.financeHubSource
        && node.dataset.aeLabel === root.dataset.financeLabel && node.dataset.aeSource === evidence.dataset.aeSource
        && node.dataset.aeSourceAvailable === 'true' && node.dataset.aeAvailable === 'true');
    }
    if (['Mail','Community'].includes(route)) {
      const workspace = unique(target, '.communications-hub[data-hub-source]');
      return workspace?.dataset.hubContext === root.dataset.financeContext && workspace.dataset.hubSource === root.dataset.financeHubSource && workspace.dataset.hubMode === root.dataset.financeMode;
    }
    return false;
  }
  function admit(control) {
    const root = owner(), physical = control.closest('#finance-preview'), local = control.closest(panelSelector);
    if (!control.isConnected || !available || !valid(current) || physical && physical !== root || local && local !== current
      || !control.getClientRects().length || control.disabled || control.closest('[hidden], [inert], [aria-hidden="true"]')) return false;
    if (control.matches('[data-workforce-anchor-action="route-leave"]')) {
      const workforce = unique(document, '#identity-workforce-service');
      return workforce?.contains(control) && host.contains(workforce) && workforce.dataset.accountContext === root.dataset.financeContext
        && workforce.dataset.accountMode === root.dataset.financeMode && workforce.dataset.accountAvailable === 'true';
    }
    return !!local || !!physical || control.closest('.console-app') === unique(document, '.console-app');
  }
  function settleOwnHash() {
    if (!available || !state || state.key !== key(current) || !valid(current)) return;
    const tab = financeTab(byId(state.tab));
    const target = tab && byId(tab.getAttribute('aria-controls'));
    if (target) reveal(target, false);
  }
"##;
