pub(super) const SCRIPT: &str = r##"
(() => {
  const host = document.getElementById('dashboard-island-root');
  let current = null, pending = null, settling = false, restoring = false;
  const owner = () => { const root = document.getElementById('finance-preview'); return root && host?.contains(root) ? root : null; };
  const normalize = value => String(value ?? '').normalize('NFC').trim().toLocaleLowerCase();
  const vendors = root => [...root.querySelectorAll('[data-finance-vendor]')];
  const details = root => [...root.querySelectorAll('[data-finance-detail]')];
  function facts(root) {
    const scene = root.querySelector('#finance-commercial-service')?.cloneNode(true);
    if (!scene) return null;
    scene.querySelectorAll('#finance-panel-leave, #leave-time, #payroll-finance-preview, [data-finance-projected]').forEach(node => node.remove());
    const projected = new Set(['aria-selected', 'aria-disabled', 'aria-current', 'tabindex', 'hidden', 'open', 'data-tablist-wired']);
    return JSON.stringify([scene.textContent, [...scene.querySelectorAll('*')].map(node => [node.tagName, [...node.attributes]
      .filter(attr => !projected.has(attr.name) && (attr.name !== 'disabled' || node.dataset.financeUnavailable === 'true'))
      .map(attr => [attr.name, attr.name === 'class' ? attr.value.split(/\s+/).filter(token => token !== 'active').join(' ') : attr.name === 'style' ? node.style.cssText : attr.value])
      .filter(([name, value]) => name !== 'class' || value !== '').sort(([a], [b]) => a.localeCompare(b))])]);
  }
  function valid(root) {
    try {
      if (!root.querySelector('[data-finance-availability]') || !root.querySelector('#finance-unavailable-reason') || !root.querySelector('[data-finance-route-status]') || !['ssr', 'island'].includes(root.dataset.financeMode)) return false;
      const source = JSON.parse(root.dataset.financeSource), scope = JSON.parse(root.dataset.financeScope), expected = document.createElement('div');
      if (source.length !== 5 || !['tenant-admin', 'corporate-office', 'healthcare-clinician'].includes(source[0])
        || source[0] !== root.dataset.financeContext || source[1] !== root.dataset.financeLabel || source[3] !== root.dataset.financeHubSource
        || JSON.stringify(source[2]) !== JSON.stringify(scope) || typeof source[4] !== 'string'
        || !['true', 'false'].includes(root.dataset.financeNative) || Object.keys(scope).length !== 3
        || ['commercial_examples', 'finops_examples', 'filing_examples'].some(name => scope[name] !== (source[0] !== 'healthcare-clinician'))
        || ['commercial_examples', 'finops_examples', 'filing_examples'].some(name => typeof scope[name] !== 'boolean')) return false;
      let body = source[4];
      if (root.dataset.financeNative === 'false') for (const [native, compat] of [['finance-panel-ledger','ledger-preview'],['finance-panel-vendors','vendors-spend'],['finance-panel-billing','billing-tax'],['finance-panel-leave','leave-time']]) for (const attribute of ['id', 'aria-controls', 'href']) { const prefix = attribute === 'href' ? '#' : ''; body = body.replaceAll(`${attribute}="${prefix}${native}"`, `${attribute}="${prefix}${compat}"`); }
      expected.innerHTML = body;
      return facts(root) !== null && facts(root) === facts(expected) && (scope.commercial_examples ? vendors(root).length === 3 && root.querySelectorAll('[data-finance-journal]').length === 4 : !vendors(root).length && !root.querySelector('[data-finance-journal]'));
    } catch { return false; }
  }
  function remember() {
    if (!current?.root.isConnected) return;
    current.open = new Set(details(current.root).filter(node => node.open).map(node => node.dataset.financeDetail));
    current.scroll = new Map([...current.root.querySelectorAll('[data-finance-scroll]')].map(node => [node.dataset.financeScroll, [node.scrollLeft, node.scrollTop]]));
  }
  function restoreScroll() {
    current.scroll.forEach(([left, top], id) => { const node = [...current.root.querySelectorAll('[data-finance-scroll]')].find(node => node.dataset.financeScroll === id); if (node) { node.scrollLeft = left; node.scrollTop = top; } });
  }
  function inViewport(node) {
    const box = node.getBoundingClientRect();
    return box.width > 0 && box.height > 0 && box.bottom > 0 && box.top < innerHeight && box.right > 0 && box.left < innerWidth;
  }
  function mirror() {
    if (!current) return;
    if (current.restoreTab && current.available && !restoring && document.hasFocus()) {
      const desired = current.restoreTab, tab = current.root.querySelector(`#finance-tab-${desired}`), saved = current.focus, active = document.activeElement;
      if (tab && !tab.disabled) {
        restoring = true; if (tab.getAttribute('aria-selected') !== 'true') tab.click(); restoring = false; current.focus = saved;
        if (active?.isConnected && active !== document.body && active !== tab && document.activeElement === tab) active.focus({ preventScroll: true });
        else if (saved && !saved.node.isConnected && document.activeElement === tab) current.restoredTabFocus = tab;
        if (tab.getAttribute('aria-selected') === 'true') { current.tab = desired; current.restoreTab = null; }
      }
    }
    if (!current.restoreTab) current.tab = current.root.querySelector('.finance-tabs [role="tab"][aria-selected="true"]')?.dataset.financeTab;
    current.root.querySelectorAll('[data-finance-side]').forEach(link => {
      const tab = current.root.querySelector(`#finance-tab-${link.dataset.financeSide}`), selected = tab?.getAttribute('aria-selected') === 'true';
      if (selected) link.setAttribute('aria-current', 'page'); else link.removeAttribute('aria-current');
      if (link.classList.contains('active') !== selected) link.classList.toggle('active', selected);
    });
  }
  function projectRails() {
    for (const slot of document.querySelectorAll('[data-finance-rail-slot]')) {
      const group = slot.dataset.financeRailSlot, template = current.available && current.root.querySelector(`template[data-finance-rail-template="${group}"]`);
      const body = template?.innerHTML || '';
      if (slot.innerHTML !== body) slot.innerHTML = body;
    }
  }
  function projectChildren() {
    for (const scene of document.querySelectorAll('[data-finance-owned="filing"], [data-finance-owned="finops"]')) {
      const available = current.available && host.contains(scene) && scene.dataset.financeOwnerSource === current.root.dataset.financeSource
        && scene.dataset.financeContext === current.root.dataset.financeContext && current.scope[`${scene.dataset.financeOwned}_examples`] === true;
      scene.dataset.financeAvailable = String(available); scene.querySelectorAll('a, summary').forEach(node => node.setAttribute('aria-disabled', String(!available)));
    }
  }
  function apply() {
    const root = current.root, rows = vendors(root), query = normalize(current.query); let count = 0;
    for (const row of rows) {
      row.hidden = !normalize(row.dataset.financeSearchText).includes(query);
      if (row.hidden) { const detail = row.querySelector('details'); if (detail) detail.open = false; current.open.delete(row.dataset.financeVendor); } else count++;
    }
    const input = root.querySelector('[data-vendor-search]'); if (input) input.value = current.query;
    const status = root.querySelector('[data-finance-vendor-status]'); if (status) status.textContent = `${count} of ${rows.length} sample vendors shown`;
    const empty = root.querySelector('[data-finance-vendor-empty]'); if (empty) empty.hidden = count !== 0;
    root.dataset.financeAvailable = String(current.available);
    const notice = root.querySelector('[data-finance-availability]'); if (notice) notice.textContent = `${current.available ? 'Observed' : 'Retained'} anonymous sample: ${root.dataset.financeLabel}. ${current.scope.commercial_examples ? 'Commercial examples only' : 'Commercial, FinOps and Filing examples omitted; preserved Payroll and Leave samples remain'}. ${current.available ? 'Explicit sample source only' : 'Requested sample pending, unavailable or mismatched; Finance controls disabled'}. No tenant or role is verified. Live financial readiness unknown.`;
    root.querySelectorAll('[data-vendor-search], #finance-tab-ledger, #finance-tab-vendors, #finance-tab-billing').forEach(node => { node.disabled = !current.available; });
    root.querySelectorAll('[data-finance-detail] summary, [data-finance-side], [data-finance-route], [data-finance-channel], .vendors-trust-board a, .billing-trust-board a, .ledger-trust-board a').forEach(node => { node.setAttribute('aria-disabled', String(!current.available)); });
    projectChildren();
    projectRails(); mirror();
  }
  function restoreFocus() {
    const saved = current.focus;
    if (!saved || saved.node.isConnected || !current.available || current.restoreTab) return;
    if (!document.hasFocus() || location.href !== saved.url) { current.focus = null; return; }
    const active = document.activeElement;
    if (active && active !== document.body && active.isConnected && active !== current.restoredTabFocus) { current.focus = null; return; }
    const target = [...current.root.querySelectorAll('[data-finance-focus]')].find(node => node.dataset.financeFocus === saved.token && !node.disabled && node.getClientRects().length);
    current.focus = null; current.restoredTabFocus = null;
    if (target) {
      target.focus({ preventScroll: true });
      if (document.activeElement === target) {
        current.focus = { ...saved, node: target };
        if (saved.visible && !inViewport(target)) { target.scrollIntoView({ block: 'center', inline: 'nearest' }); restoreScroll(); }
        current.focus.visible = inViewport(target);
      }
    }
  }
  function synchronize() {
    const root = owner(); if (!root) { if (current) { current.available = false; projectRails(); projectChildren(); } return; }
    const key = JSON.stringify([root.dataset.financeSource, facts(root)]), ready = root.dataset.financeSourceAvailable === 'true' && valid(root), mode = root.dataset.financeMode;
    if (current?.root === root && current.key === key && current.available === ready && current.mode === mode) { projectChildren(); mirror(); restoreFocus(); return; }
    remember(); const previous = current, same = previous?.key === key;
    if (!same) pending = null;
    let scope; try { const parsed = JSON.parse(root.dataset.financeScope); scope = parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {}; } catch { scope = {}; }
    current = { root, key, available: ready, mode, scope, query: same ? previous.query : '', open: same ? previous.open : new Set(), scroll: same ? previous.scroll : new Map(), focus: same ? previous.focus : null, tab: same ? previous.tab : null, restoreTab: same && previous.root !== root ? previous.tab : same ? previous.restoreTab : previous && ready && scope.commercial_examples === true ? 'ledger' : null };
    details(root).forEach(node => { node.open = current.open.has(node.dataset.financeDetail); });
    const reset = root.querySelector('[data-finance-reset]'); if (reset) reset.hidden = same || !previous;
    apply();
    restoreScroll();
    restoreFocus();
    if (current.restoreTab) { const bound = current; requestAnimationFrame(() => { if (current === bound) { mirror(); restoreFocus(); } }); }
    if (pending && current.mode === 'island' && current.available) { const request = pending; pending = null; if (request.key === key && location.href === request.url) openChannel(request.channel); }
  }
  function target(event) {
    if (!(event.target instanceof Element) || !event.target.isConnected) return null;
    const root = owner(), scene = event.target.closest('[data-finance-owned]');
    if (!root || !(root.contains(event.target) || scene && host.contains(scene) && ['filing', 'finops'].includes(scene.dataset.financeOwned))) return null;
    synchronize(); return current?.root === root ? event.target : null;
  }
  function workspace() {
    const hub = document.getElementById('work-hub'), root = hub?.querySelector('.communications-hub[data-hub-source]');
    return root && host.contains(hub) && root.dataset.hubContext === current.root.dataset.financeContext
      && root.dataset.hubSource === current.root.dataset.financeHubSource && root.dataset.hubMode === current.mode ? root : null;
  }
  function channelReady(root, channel) {
    const tab = root?.querySelector(`#hub-tab-${channel.toLowerCase()}`), composer = root?.querySelector('.hub-composer textarea');
    return tab?.getAttribute('aria-selected') === 'true' && (current.mode === 'ssr' || !tab.disabled && composer && !composer.disabled)
      && root.querySelector('[data-comms-product-board]')?.dataset.commsBoardSurface === channel
      && root.querySelector('.hub-list') && root.querySelector('.hub-detail') && composer;
  }
  function openChannel(channel) {
    if (!['Mail', 'Community', 'Messenger'].includes(channel) || !current.available) return false;
    let root = workspace(); if (!root) return false;
    if (channelReady(root, channel)) return true;
    if (current.mode === 'island') { const tab = root.querySelector(`#hub-tab-${channel.toLowerCase()}`); if (!tab || tab.disabled) return false; tab.click(); }
    else {
      const template = [...current.root.querySelectorAll('[data-finance-hub-template]')].find(node => node.dataset.financeHubTemplate === channel), replacement = template?.content.firstElementChild?.cloneNode(true);
      if (!replacement || replacement.dataset.hubContext !== current.root.dataset.financeContext || replacement.dataset.hubSource !== current.root.dataset.financeHubSource
        || replacement.dataset.hubMode !== current.mode || !channelReady(replacement, channel)) return false;
      root.replaceWith(replacement);
    }
    const key = current.key;
    requestAnimationFrame(() => { if (current?.key === key && current.available) current.root.querySelector('[data-finance-route-status]').textContent = channelReady(workspace(), channel) ? `${channel} existing sample opened; no draft, message or receipt created.` : `${channel} sample unavailable; no output created.`; });
    return true;
  }
  function revealSSR(destination) {
    const panel = destination.closest('[role="tabpanel"]');
    if (panel) {
      const tab = document.getElementById(panel.getAttribute('aria-labelledby')), group = tab?.closest('[role="tablist"]');
      if (!tab || tab.disabled || tab.getAttribute('aria-controls') !== panel.id || tab.parentElement !== group || group.hasAttribute('data-native-hub-tabs') || group.classList.contains('hub-tabs')) return false;
      tab.click(); if (tab.getAttribute('aria-selected') !== 'true' || !panel.classList.contains('active')) return false;
    }
    if (document.hasFocus()) { if (!destination.hasAttribute('tabindex')) destination.setAttribute('tabindex', '-1'); destination.focus({ preventScroll: true }); destination.scrollIntoView({ block: 'start' }); }
    return true;
  }
  document.addEventListener('input', event => { const input = target(event)?.closest('[data-vendor-search]'); if (input && current.available && !input.disabled) { current.query = input.value; apply(); } });
  document.addEventListener('click', event => {
    const scene = event.target instanceof Element && event.target.closest('[data-finance-owned]');
    if (scene && host?.contains(scene)) { synchronize(); if (!current?.available || scene.dataset.financeOwnerSource !== current.root.dataset.financeSource || scene.dataset.financeContext !== current.root.dataset.financeContext || current.scope[`${scene.dataset.financeOwned}_examples`] !== true) { event.preventDefault(); return; } }
    const control = target(event); if (!control) return;
    if (control.closest('#finance-panel-leave, #leave-time, #payroll-finance-preview, #finance-tab-leave') || control.closest('a[href="#finance-panel-leave"], a[href="#leave-time"]')) return;
    if (!current.available || control.closest('[data-finance-unavailable]')) { event.preventDefault(); return; }
    const link = control.closest('a[href^="#"]'); if (!link) return;
    const destination = document.getElementById(link.getAttribute('href').slice(1));
    if (!destination || link.dataset.financeChannel && !openChannel(link.dataset.financeChannel) || current.mode === 'ssr' && !revealSSR(destination)) {
      event.preventDefault(); current.root.querySelector('[data-finance-route-status]').textContent = 'Existing sample destination unavailable; no output created.'; return;
    }
    current.focus = null; current.root.querySelector('[data-finance-route-status]').textContent = 'Existing sample view opened; no financial action, draft, message or receipt created.';
    if (current.mode === 'ssr' && link.dataset.financeChannel) {
      if (location.href === link.href) event.preventDefault();
      pending = { key: current.key, channel: link.dataset.financeChannel, oldURL: location.href, url: link.href, awaitingHash: location.href !== link.href, ownPop: location.href !== link.href };
    }
  }, true);
  document.addEventListener('toggle', event => { if (target(event)?.matches('[data-finance-detail]')) remember(); }, true);
  document.addEventListener('scroll', event => {
    if (target(event)?.matches('[data-finance-scroll]')) remember();
    const saved = current?.focus;
    if (saved?.node.isConnected && document.activeElement === saved.node) saved.visible = inViewport(saved.node);
  }, true);
  document.addEventListener('focusin', event => {
    if (restoring) return;
    const control = target(event)?.closest('[data-finance-focus]');
    if (control && current.available) current.focus = { node: control, token: control.dataset.financeFocus, url: location.href, visible: inViewport(control) };
    else if (current && event.target !== document.body) current.focus = null;
  });
  for (const name of ['pointerdown', 'keydown']) document.addEventListener(name, event => {
    if (!event.isTrusted || !(event.target instanceof Element)) return;
    pending = null; if (current && event.target.closest('.finance-tabs')) current.restoreTab = null; if (current?.focus && event.target !== current.focus.node) current.focus = null;
  }, true);
  document.addEventListener('keydown', event => { const scene = event.target instanceof Element && event.target.closest('[data-finance-owned]'); if (scene && host?.contains(scene) && ['Enter', ' ', 'Spacebar'].includes(event.key)) { synchronize(); if (!current?.available || scene.dataset.financeOwnerSource !== current.root.dataset.financeSource || scene.dataset.financeContext !== current.root.dataset.financeContext || current.scope[`${scene.dataset.financeOwned}_examples`] !== true) { event.preventDefault(); return; } } if (target(event) && !current.available && ['Enter', ' ', 'Spacebar'].includes(event.key) && !event.target.closest('#finance-panel-leave, #leave-time, #payroll-finance-preview, #finance-tab-leave')) event.preventDefault(); }, true);
  window.addEventListener('blur', () => { if (current) current.focus = null; });
  window.addEventListener('hashchange', event => {
    if (current) current.focus = null;
    if (pending?.awaitingHash && event.oldURL === pending.oldURL && event.newURL === pending.url && location.href === pending.url) { pending.awaitingHash = false; pending.ownPop = false; } else pending = null;
  });
  window.addEventListener('popstate', () => { if (current) current.focus = null; if (pending?.awaitingHash && pending.ownPop && location.href === pending.url) pending.ownPop = false; else pending = null; });
  if (host) new MutationObserver(() => { if (settling) return; settling = true; queueMicrotask(() => { settling = false; synchronize(); }); })
    .observe(host, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ['data-finance-source', 'data-finance-context', 'data-finance-label', 'data-finance-scope', 'data-finance-hub-source', 'data-finance-source-available', 'data-finance-mode', 'data-finance-native', 'data-finance-record', 'data-finance-search-text', 'data-finance-vendor', 'data-finance-journal', 'data-finance-detail', 'data-finance-focus', 'data-finance-scroll', 'data-finance-unavailable', 'data-finance-route', 'data-finance-channel', 'data-finance-side', 'data-finance-action', 'data-finance-command-action', 'data-trust-proof-action', 'data-finance-tab', 'data-finance-panel', 'data-finance-availability', 'data-finance-route-status', 'data-finance-owner-source', 'data-finance-owned', 'id', 'href', 'role', 'aria-controls', 'aria-labelledby', 'aria-label', 'style', 'class', 'disabled', 'aria-selected'] });
  synchronize();
})();
"##;
