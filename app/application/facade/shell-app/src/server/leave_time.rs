mod source;

pub(super) fn script() -> String {
    [source::PRELUDE, CONTROLLER].concat()
}

const CONTROLLER: &str = r##"
  const status = text => { const node = current?.querySelector('[data-leave-status]'); if (node) node.textContent = text; };
  function focus(node) {
    if (!document.hasFocus() || !node?.isConnected || node.disabled || node.closest('[hidden], [inert], [aria-hidden="true"]') || !node.getClientRects().length) return;
    if (!node.hasAttribute('tabindex') && !node.matches('button,input,select,summary,a')) node.setAttribute('tabindex', '-1');
    node.focus({ preventScroll: true });
    node.scrollIntoView({ block: 'nearest' });
  }
  function reveal(target, moveFocus) {
    const owner = target.closest('[role="tabpanel"]');
    const tab = owner && byId(owner.getAttribute('aria-labelledby'));
    const list = tab?.closest('[role="tablist"]');
    if (tab?.getAttribute('aria-controls') === owner?.id && list) {
      if (list.hasAttribute('data-tablist-wired')) {
        if (tab.getAttribute('aria-selected') !== 'true') {
          const previousFocus = document.activeElement;
          forwarding = true;
          try { tab.click(); } finally { forwarding = false; }
          if (!moveFocus && previousFocus?.isConnected && previousFocus !== document.body) previousFocus.focus({ preventScroll: true });
        }
      } else {
        for (const candidate of list.querySelectorAll('[role="tab"]')) {
          const selected = candidate === tab;
          candidate.setAttribute('aria-selected', String(selected));
          candidate.setAttribute('tabindex', selected ? '0' : '-1');
          candidate.classList.toggle('active', selected);
          byId(candidate.getAttribute('aria-controls'))?.classList.toggle('active', selected);
        }
      }
    }
    if (owner && (tab?.getAttribute('aria-selected') !== 'true' || !owner.classList.contains('active'))) return false;
    if (moveFocus) focus(target instanceof HTMLDetailsElement ? target.querySelector('summary') : target);
    return !moveFocus || target.getClientRects().length > 0;
  }
  function coverage() {
    const select = current.querySelector('#leave-sample-backup');
    const selected = select.value === 'none' ? null : select.selectedOptions[0].textContent;
    current.querySelector('#leave-coverage-preview').textContent = selected
      ? `Local coverage preview: ${selected}; no assignment or schedule change made.`
      : 'No sample backup selected; no coverage assignment made.';
  }
  function filter() {
    const query = current.querySelector('#leave-sample-search').value.trim().toLocaleLowerCase();
    let count = 0;
    for (const row of current.querySelectorAll('[data-leave-item]')) {
      row.hidden = !row.textContent.toLocaleLowerCase().includes(query);
      if (!row.hidden) count++;
    }
    current.querySelector('#leave-sample-count').textContent = `${count} sample ${count === 1 ? 'item' : 'items'}${query ? ' matching your search' : ''}`;
  }
  const financeTabs = () => byId(current?.getAttribute('aria-labelledby'))?.closest('.finance-tabs[role="tablist"]');
  function financeTab(node) {
    const target = node && byId(node.getAttribute('aria-controls'));
    return node?.matches('[role="tab"]') && node.closest('[role="tablist"]') === financeTabs()
      && target?.getAttribute('aria-labelledby') === node.id ? node : null;
  }
  function locator(node) {
    if (!node || (!current.contains(node) && !financeTab(node))) return null;
    if (node.id) return { id: node.id };
    for (const attr of ['data-finance-action','data-finance-anchor-action','data-leave-close']) {
      if (node.hasAttribute(attr)) {
        const value = node.getAttribute(attr);
        return { attr, value, index: [...current.querySelectorAll(`[${attr}="${value}"]`)].indexOf(node) };
      }
    }
    return null;
  }
  function locate(token) {
    if (!token) return null;
    if (token.id) {
      const node = byId(token.id);
      return locator(node) ? node : null;
    }
    return current.querySelectorAll(`[${token.attr}="${token.value}"]`)[token.index];
  }
  function remember(preserveFocus = false) {
    if (!current?.isConnected || !available) return;
    const active = document.activeElement;
    const ownedFocus = locator(active);
    const retainFocus = preserveFocus && !ownedFocus;
    state = {
      key: key(current), url: location.href, query: current.querySelector('#leave-sample-search').value,
      backup: current.querySelector('#leave-sample-backup').value,
      details: [...current.querySelectorAll('[data-leave-detail]')].map(d => [d.dataset.leaveDetail, d.open]),
      selected: current.classList.contains('active'), tab: financeTabs()?.querySelector('[role="tab"][aria-selected="true"]')?.id,
      focus: retainFocus ? state?.focus ?? null : ownedFocus,
      focusNode: retainFocus ? state?.focusNode ?? active : active, status: current.querySelector('[data-leave-status]').textContent,
    };
  }
  function restore() {
    const next = panel();
    const wasAvailable = available;
    available = valid(next);
    for (const anchor of all(document, 'a').filter(isOwned)) if (!boundAnchors.has(anchor)) {
      anchor.addEventListener('click', event => { if (!admit(anchor)) event.preventDefault(); });
      boundAnchors.add(anchor);
    }
    for (const candidate of all(document, panelSelector)) normalize(candidate, available && candidate === next);
    if (!available) return;
    if (next === current && state?.key === key(next)) { if (!wasAvailable && state) status(state.status); return; }
    const previous = current;
    const saved = state;
    current = next;
    const matching = previous && saved?.key === key(next);
    if (matching) {
      next.querySelector('#leave-sample-search').value = saved.query;
      next.querySelector('#leave-sample-backup').value = saved.backup;
      for (const [kind, open] of saved.details) next.querySelector(`[data-leave-detail="${kind}"]`).open = open;
      status(saved.status);
      const active = document.activeElement;
      const restoreFocus = saved.focus && document.hasFocus() && location.href === saved.url && (active === document.body || !active?.isConnected || active === saved.focusNode || host?.contains(active));
      const selectedTab = financeTab(byId(saved.tab));
      const selectedPanel = selectedTab && byId(selectedTab.getAttribute('aria-controls'));
      if (selectedPanel) reveal(selectedPanel, false);
      if (restoreFocus) focus(locate(saved.focus));
    } else {
      origins = {}; ownHash = null;
      if (previous) {
        const search = next.querySelector('#leave-sample-search'), backup = next.querySelector('#leave-sample-backup');
        search.value = search.defaultValue;
        backup.value = [...backup.options].find(option => option.defaultSelected)?.value ?? backup.options[0]?.value ?? '';
        for (const detail of next.querySelectorAll('[data-leave-detail]')) detail.open = false;
        status('Sample preview reset for changed tenant, role, or source; no live data connected.');
      }
      let id;
      try { id = decodeURIComponent(location.hash.slice(1)); } catch { id = ''; }
      const target = byId(id);
      if (target && next.contains(target)) {
        reveal(target, false);
        // Only initial direct entry opens details; a context/source reset stays closed.
        if (!previous && target instanceof HTMLDetailsElement) target.open = true;
        if (!previous && document.activeElement === document.body) {
          focus(target instanceof HTMLDetailsElement ? target.querySelector('summary') : next);
        }
      }
    }
    coverage(); filter(); remember();
  }
  function navigate(id) {
    if (location.hash === `#${id}`) return;
    const oldURL = location.href; location.hash = id;
    ownHash = { oldURL, url: location.href, ownPop: true };
  }
  function enter(nativeTab) {
    if (!reveal(current, false)) { status('Sample Leave destination unavailable; no action performed.'); return; }
    const tab = byId(current.getAttribute('aria-labelledby'));
    focus(nativeTab && tab?.getAttribute('aria-controls') === current.id ? tab : current);
    status('Sample leave and time preview opened locally; no approval, assignment, lock, or payroll effect.');
    if (!nativeTab) navigate(current.id);
    remember();
  }
  const routes = {
    'route-workflow': 'workflow-studio', 'route-cloud': 'cloud-ops-cockpit',
    'route-policy': 'cockpit-panel-policy', 'route-audit': 'resource-panel-audit',
    'route-ledger': 'finance-panel-ledger', 'route-evidence': 'evidence-spine',
    'route-mail': 'Mail', 'route-community': 'Community',
  };
  document.addEventListener('click', event => {
    if (forwarding || !(event.target instanceof Element) || event.defaultPrevented || event.button !== 0
        || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    const control = event.target.closest('button,a');
    if (!control?.isConnected) { if (control?.closest(panelSelector)) event.preventDefault(); return; }
    const inbound = control.matches(inboundSelector);
    const entry = isEntry(control);
    if (!inbound && !entry && !panel()?.contains(control) && !control.closest('.finance-tabs [role="tab"]')) return;
    restore();
    if (!current || !available || !control.getClientRects().length || control.disabled) { event.preventDefault(); return; }
    if (inbound || entry) { enter(control.matches('[data-finance-tab="leave"]')); return; }
    if (control.closest('.finance-tabs [role="tab"]')) {
      const tab = financeTab(control);
      const target = tab && byId(tab.getAttribute('aria-controls'));
      if (target) { reveal(target, false); remember(); }
      return;
    }
    if (!current.contains(control)) return;
    const action = control.dataset.financeAction || control.dataset.financeAnchorAction;
    const kind = { 'approve-leave':'request', 'reassign-time':'coverage', 'lock-timesheets':'cutoff' }[action];
    if (kind) {
      const details = current.querySelector(`[data-leave-detail="${kind}"]`);
      origins[kind] = locator(control);
      details.open = true;
      focus(kind === 'coverage' ? current.querySelector('#leave-sample-backup') : details.querySelector('summary'));
      status(`Sample ${kind} opened locally; no approval, assignment, lock, receipt, or payroll effect.`);
    } else if (control.dataset.leaveClose) {
      const kind = control.dataset.leaveClose;
      const detail = current.querySelector(`[data-leave-detail="${kind}"]`);
      detail.open = false;
      focus(locate(origins[kind]) || detail.querySelector('summary'));
    } else if (routes[action]) {
      const route = routes[action];
      const channel = ['Mail','Community'].includes(route);
      const target = route === 'finance-panel-ledger'
        ? unique(owner(), '[data-finance-panel="ledger"]')
        : unique(document, `#${channel ? 'work-hub' : route}`);
      if (!destinationReady(target, route)) { status('Sample destination unavailable; no action performed.'); remember(); return; }
      if (channel) {
        // Consume the accepted complete channel renderer through its existing native control.
        const policy = unique(document, '#cockpit-panel-policy');
        const native = unique(policy, `[data-policy-channel="${route}"]`);
        if (!native || policy.dataset.policyContext !== owner().dataset.financeContext || policy.dataset.policyMode !== owner().dataset.financeMode) { status('Sample channel unavailable; no message sent or posted.'); remember(); return; }
        native.click();
        if (!destinationReady(target, route) || unique(target, '[data-comms-product-board]')?.dataset.commsBoardSurface !== route) { status('Sample channel unavailable; no message sent or posted.'); remember(); return; }
      }
      if (!reveal(target, true)) { status('Sample destination unavailable; no action performed.'); remember(); return; }
      status(`${control.textContent.trim()} sample preview opened locally; no workflow, send, publication, evidence, or payroll effect.`);
      navigate(target.id);
    }
    remember();
  });
  for (const name of ['input','change','toggle','focusin']) document.addEventListener(name, event => {
    if (!current?.isConnected || !available || !valid(current) || !event.target.isConnected || (!current.contains(event.target) && !(name === 'focusin' && locator(event.target)))) return;
    if (event.target.id === 'leave-sample-search') filter();
    if (event.target.id === 'leave-sample-backup') coverage();
    remember(name !== 'focusin');
  }, true);
  document.addEventListener('keydown', event => {
    if (current?.isConnected && available && event.target instanceof Element
        && event.target.closest('.finance-tabs [role="tab"]')) remember();
  });
  document.addEventListener('focusin', event => {
    if (state && current?.isConnected && !locator(event.target) && event.target !== document.body) {
      state.focus = null;
      state.focusNode = event.target;
    }
  });
  for (const name of ['hashchange','popstate']) window.addEventListener(name, event => {
    if (ownHash && location.href === ownHash.url) {
      if (name === 'popstate' && ownHash.ownPop) { ownHash.ownPop = false; settleOwnHash(); return; }
      if (name === 'hashchange' && event.oldURL === ownHash.oldURL && event.newURL === ownHash.url) { ownHash = null; settleOwnHash(); return; }
    }
    ownHash = null;
    if (state) state.focus = null;
    restore();
    let id;
    try { id = decodeURIComponent(location.hash.slice(1)); } catch { return; }
    const target = byId(id);
    if (available && current && target && (target === current || current.contains(target))) { reveal(target, true); remember(); }
  });
  document.addEventListener('click', event => {
    if (!(event.target instanceof Element)) return;
    const control = event.target.closest('button,a,input,select,summary');
    if (!control) return;
    const owned = isOwned(control), relatedTab = control.matches('.finance-tabs [role="tab"]') && control.closest('#finance-preview') === owner();
    if (!owned && !relatedTab) return;
    restore();
    if (owned && !admit(control)) { event.preventDefault(); event.stopPropagation(); }
  }, true);
  for (const name of ['pointerdown','keydown']) document.addEventListener(name, event => {
    if (name === 'keydown' && event.target instanceof Element && event.target.matches('.finance-tabs [role="tab"]') && event.target.closest('#finance-preview') === owner()) restore();
    if (event.isTrusted && state && event.target !== state.focusNode && !locator(event.target)) { state.focus = null; state.focusNode = event.target; }
  }, true);
  window.addEventListener('blur', () => { if (state) state.focus = null; });
  if (host) new MutationObserver(restore).observe(host, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ['data-finance-source','data-finance-source-available','data-finance-available','data-finance-context','data-finance-label','data-finance-hub-source','data-finance-scope','data-finance-mode','data-finance-native','data-leave-context','data-leave-source','data-leave-mode','data-sample-context-requested','data-sample-context-displayed','aria-busy','data-fragment-navigation','id','href','role','aria-controls','aria-labelledby'] });
  restore();
})();
"##;

#[cfg(test)]
#[path = "leave_time_test.rs"]
mod tests;
