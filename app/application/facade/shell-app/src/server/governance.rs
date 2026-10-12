pub(super) const SCRIPT: &str = r#"
(() => {
  const host = document.getElementById('dashboard-island-root');
  const ownedRoots = new WeakSet();
  let current = null, pending = null, settling = false;
  const owner = () => {
    const roots = document.querySelectorAll('#governance-preview');
    if (roots.length !== 1) return null;
    const root = roots[0];
    return root?.dataset.governanceSource && host?.contains(root) ? root : null;
  };
  const rows = root => [...root.querySelectorAll('[data-logic-row]')];
  const details = root => [...root.querySelectorAll('[data-logic-detail]')];
  const risks = root => [...root.querySelectorAll('[data-governance-risk]')];
  const calendar = root => [...root.querySelectorAll('[data-gov-calendar-cell]')];
  const normalize = value => value.normalize('NFC').trim().toLocaleLowerCase();
  function facts(root) {
    return JSON.stringify([root.querySelector('[data-governance-workflow-source]')?.dataset.governanceWorkflowSource, rows(root).map(row => [row.dataset.logicId, row.dataset.logicRecord, row.dataset.logicCategory,
      row.dataset.logicState, row.dataset.logicSearchText, row.textContent]),
      risks(root).map(pin => [pin.dataset.governanceRisk, pin.dataset.riskTitle, pin.dataset.riskDetail, pin.dataset.riskOwner, pin.dataset.riskScore, pin.textContent]),
      calendar(root).map(cell => [cell.dataset.govCalendarCell, cell.dataset.govCalendarDescription, cell.textContent])]);
  }
  function valid(root) {
    try {
      const source = JSON.parse(root.dataset.governanceSource), expected = document.createElement('div');
      if (source.length !== 4 || source[0] !== root.dataset.governanceContext || source[1] !== root.dataset.governanceLabel
        || source[2] !== root.dataset.governanceHubSource || typeof source[3] !== 'string') return false;
      expected.innerHTML = source[3];
      if (facts(root) !== facts(expected)) return false;
      if (root.dataset.governanceVisible !== String(['tenant-admin', 'corporate-office'].includes(source[0]))) return false;
      if (source[0] === 'healthcare-clinician') return true;
      const ids = new Set();
      for (const row of rows(root)) {
        const record = JSON.parse(row.dataset.logicRecord);
        if (!record.id || !record.owner || ids.has(record.id) || Object.values(record).some(value => typeof value !== 'string')
          || row.dataset.logicId !== record.id || row.dataset.logicCategory !== record.category || row.dataset.logicState !== record.state) return false;
        ids.add(record.id);
      }
      return ids.size === 7 && risks(root).length === 5 && calendar(root).length === 10;
    } catch { return false; }
  }
  function remember() {
    if (!current?.root.isConnected || !current.visible) return;
    current.open = new Set(details(current.root).filter(node => node.open).map(node => node.dataset.logicDetail));
    current.scroll = new Map([...current.root.querySelectorAll('[data-governance-scroll]')].map(node => [node.dataset.governanceScroll, [node.scrollLeft, node.scrollTop]]));
  }
  function apply() {
    const root = current.root;
    document.querySelectorAll('[data-governance-rail]').forEach(link => { link.hidden = !current.visible || !current.available; });
    if (!current.visible) {
      root.querySelector('[data-governance-availability]').textContent = `${current.ready ? 'Observed' : 'Retained'} anonymous sample: ${root.dataset.governanceLabel}; executive Governance examples omitted for the clinician sample. ${current.ready ? 'Anonymous example source only' : 'Requested sample pending, unavailable or mismatched'}. No tenant or role verified; live Governance unavailable.`;
      return;
    }
    let count = 0;
    for (const row of rows(root)) {
      row.hidden = !((current.filter === 'all' || row.dataset.logicCategory === current.filter
        || current.filter === 'attention' && ['blocked', 'at-risk'].includes(row.dataset.logicState))
        && normalize(row.dataset.logicSearchText).includes(normalize(current.query)));
      if (row.hidden) { row.querySelector('details').open = false; current.open.delete(row.dataset.logicId); }
      else count++;
    }
    root.querySelector('[data-logic-search]').value = current.query;
    root.querySelectorAll('[data-logic-filter]').forEach(button => {
      const selected = button.dataset.logicFilter === current.filter;
      button.setAttribute('aria-pressed', String(selected)); button.classList.toggle('active', selected);
    });
    root.querySelector('[data-logic-status]').textContent = `${count} of ${rows(root).length} sample logics shown · ${current.filter}`;
    root.querySelector('[data-logic-empty]').hidden = count !== 0;
    risks(root).forEach(pin => {
      const selected = pin.dataset.governanceRisk === current.risk;
      pin.setAttribute('aria-pressed', String(selected)); pin.classList.toggle('selected', selected);
      if (selected) for (const field of ['id', 'title', 'detail', 'owner', 'score']) {
        root.querySelector(`[data-risk-peek-${field}]`).textContent = field === 'id' ? pin.dataset.governanceRisk : pin.dataset[`risk${field[0].toUpperCase()}${field.slice(1)}`];
      }
    });
    root.querySelector('[data-governance-risk-status]').textContent = `Sample ${current.risk} selected; no live score or owner verified.`;
    calendar(root).forEach(cell => {
      const selected = cell.dataset.govCalendarCell === current.calendar;
      cell.setAttribute('aria-pressed', String(selected)); cell.classList.toggle('is-selected', selected);
      if (selected) root.querySelector('[data-governance-calendar-status]').textContent = cell.dataset.govCalendarDescription;
    });
    root.querySelector('[data-governance-availability]').textContent = `${current.available ? 'Observed' : 'Retained'} anonymous sample: ${root.dataset.governanceLabel}; ${current.available ? 'local examples only' : 'requested sample or native source unavailable; local controls disabled'}. No tenant or role verified. Live Governance unavailable.`;
    root.querySelectorAll('button, input').forEach(control => { control.disabled = !current.available || control.dataset.governanceUnavailable === 'true'; });
    root.querySelectorAll('summary, [data-governance-route]').forEach(control => { control.setAttribute('aria-disabled', String(!current.available)); });
  }
  function restoreFocus() {
    const saved = current.focus;
    if (!saved || saved.node.isConnected || !current.available) return;
    if (!document.hasFocus() || location.href !== saved.url) { current.focus = null; return; }
    const active = document.activeElement;
    if (active && active !== document.body && active.isConnected) { current.focus = null; return; }
    const target = [...current.root.querySelectorAll('[data-governance-focus]')].find(node => node.dataset.governanceFocus === saved.token && !node.disabled && node.getClientRects().length);
    current.focus = null;
    if (target) { target.focus({ preventScroll: true }); if (document.activeElement === target) current.focus = { ...saved, node: target }; }
  }
  function projectUnavailable(root) {
    ownedRoots.add(root);
    root.querySelectorAll('button, input').forEach(control => { if (!control.disabled) control.disabled = true; });
    root.querySelectorAll('summary, [data-governance-route]').forEach(control => {
      if (control.getAttribute('aria-disabled') !== 'true') control.setAttribute('aria-disabled', 'true');
    });
    const status = root.querySelector('[data-governance-availability]');
    const message = 'Retained anonymous Governance sample: native source unavailable; local controls disabled. No tenant or role verified. Live Governance unavailable.';
    if (status && status.textContent !== message) status.textContent = message;
  }
  function revoke() {
    const roots = new Set(document.querySelectorAll('#governance-preview'));
    if (current) roots.add(current.root);
    current = null; pending = null;
    document.querySelectorAll('[data-governance-rail]').forEach(link => { if (!link.hidden) link.hidden = true; });
    roots.forEach(projectUnavailable);
  }
  function synchronize() {
    const root = owner();
    if (current && current.root !== root && current.root.isConnected) {
      projectUnavailable(current.root); current = null; pending = null;
    }
    if (!root || !valid(root)) { revoke(); return; }
    ownedRoots.add(root);
    const visible = root.dataset.governanceVisible === 'true', key = JSON.stringify([root.dataset.governanceSource, facts(root)]);
    const ready = root.dataset.governanceSourceAvailable === 'true', available = visible && ready && root.dataset.governanceAvailable === 'true', mode = root.dataset.governanceMode;
    if (current?.root === root && current.key === key && current.available === available && current.ready === ready && current.mode === mode) return;
    remember(); const previous = current, same = previous?.key === key;
    if (previous && !same) pending = null;
    current = { root, key, visible, ready, available, mode, query: same ? previous.query : '', filter: same ? previous.filter : 'all',
      risk: same ? previous.risk : 'RISK-042', calendar: same ? previous.calendar : null, open: same ? previous.open : new Set(),
      scroll: same ? previous.scroll : new Map(), focus: same ? previous.focus : null };
    if (visible) {
      details(root).forEach(node => { node.open = current.open.has(node.dataset.logicDetail); });
      root.querySelector('[data-governance-reset]').hidden = same || !previous;
    }
    apply();
    current.scroll.forEach(([left, top], id) => { const node = [...root.querySelectorAll('[data-governance-scroll]')].find(node => node.dataset.governanceScroll === id); if (node) { node.scrollLeft = left; node.scrollTop = top; } });
    restoreFocus();
    if (pending && current.mode === 'island' && current.available) {
      const request = pending; pending = null;
      if (request.key === key && location.href === request.url) openChannel(request.channel);
    }
  }
  function target(event) {
    if (!(event.target instanceof Element) || !event.target.isConnected) return null;
    const root = event.target.closest('#governance-preview[data-governance-source]');
    if (!root || root !== owner()) return null;
    synchronize(); return current?.root === root ? event.target : null;
  }
  function workspace() {
    const hub = document.getElementById('work-hub'), root = hub?.querySelector('.communications-hub[data-hub-source]');
    return root && host.contains(hub) && root.dataset.hubContext === current.root.dataset.governanceContext
      && root.dataset.hubSource === current.root.dataset.governanceHubSource && root.dataset.hubMode === current.mode ? root : null;
  }
  function channelReady(root, channel) {
    const tab = root?.querySelector(`#hub-tab-${channel.toLowerCase()}`), composer = root?.querySelector('.hub-composer textarea');
    return tab?.getAttribute('aria-selected') === 'true'
      && (current.mode === 'ssr' || !tab.disabled && composer && !composer.disabled)
      && root.querySelector('[data-comms-product-board]')?.dataset.commsBoardSurface === channel
      && root.querySelector('.hub-list') && root.querySelector('.hub-detail') && root.querySelector('.hub-composer textarea');
  }
  function openChannel(channel) {
    if (!['Mail', 'Community'].includes(channel) || !current.available) return false;
    let root = workspace(); if (!root) return false;
    if (channelReady(root, channel)) return true;
    if (current.mode === 'island') {
      const tab = root.querySelector(`#hub-tab-${channel.toLowerCase()}`);
      if (!tab || tab.disabled) return false; tab.click();
    } else {
      const template = [...current.root.querySelectorAll('[data-governance-hub-template]')].find(node => node.dataset.governanceHubTemplate === channel);
      const replacement = template?.content.firstElementChild?.cloneNode(true);
      if (!replacement || replacement.dataset.hubContext !== current.root.dataset.governanceContext
        || replacement.dataset.hubSource !== current.root.dataset.governanceHubSource || replacement.dataset.hubMode !== current.mode || !channelReady(replacement, channel)) return false;
      root.replaceWith(replacement);
    }
    const key = current.key;
    requestAnimationFrame(() => {
      if (current?.key === key && current.available) current.root.querySelector('[data-governance-route-status]').textContent = channelReady(workspace(), channel)
        ? `${channel} existing sample opened; no draft, message, publication or receipt created.` : `${channel} sample unavailable; no output created.`;
    });
    return true;
  }
  function revealSSR(destination) {
    const panel = destination.closest('[role="tabpanel"]');
    if (panel) {
      const tab = document.getElementById(panel.getAttribute('aria-labelledby')), group = tab?.closest('[role="tablist"]');
      if (!tab || tab.disabled || tab.getAttribute('aria-controls') !== panel.id || tab.parentElement !== group
        || group.hasAttribute('data-native-hub-tabs') || group.classList.contains('hub-tabs')) return false;
      tab.click(); if (tab.getAttribute('aria-selected') !== 'true' || !panel.classList.contains('active')) return false;
    }
    if (document.hasFocus()) { if (!destination.hasAttribute('tabindex')) destination.setAttribute('tabindex', '-1'); destination.focus({ preventScroll: true }); destination.scrollIntoView({ block: 'start' }); }
    return true;
  }
  function refuseUnavailable(event) {
    if (!(event.target instanceof Element) || event.type === 'keydown' && !['Enter', ' ', 'Spacebar'].includes(event.key)) return;
    const control = event.target.closest('button, input, summary, [data-governance-route], [data-governance-rail]');
    if (!control) return;
    let root = control.closest('#governance-preview');
    if (!root) for (let node = control.parentElement; node; node = node.parentElement) {
      if (ownedRoots.has(node)) { root = node; break; }
    }
    if (!root && !control.hasAttribute('data-governance-rail')) return;
    synchronize();
    if (!current?.available || root && root !== current.root || control.dataset.governanceUnavailable === 'true') event.preventDefault();
  }
  for (const name of ['click', 'keydown']) document.addEventListener(name, refuseUnavailable, true);
  document.addEventListener('input', event => {
    const control = target(event); if (!control?.matches('[data-logic-search]')) return;
    if (!current.available) { control.value = current.query; return; } current.query = control.value; apply();
  });
  document.addEventListener('click', event => {
    const control = target(event);
    if (!control || event.defaultPrevented || event.button !== 0 || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    if (!current.available) { event.preventDefault(); return; }
    const filter = control.closest('[data-logic-filter]'), risk = control.closest('[data-governance-risk]'), cell = control.closest('[data-gov-calendar-cell]');
    if (filter) { current.filter = filter.dataset.logicFilter; apply(); }
    else if (risk) { current.risk = risk.dataset.governanceRisk; apply(); }
    else if (cell) { current.calendar = cell.dataset.govCalendarCell; apply(); }
    else if (control.closest('[data-logic-close]')) { const detail = control.closest('details'); detail.open = false; remember(); if (document.hasFocus()) detail.querySelector('summary').focus({ preventScroll: true }); }
    else {
      const link = control.closest('[data-governance-route]'); if (!link) return;
      const destination = document.getElementById(link.getAttribute('href').slice(1));
      if (!destination || link.dataset.governanceChannel && !openChannel(link.dataset.governanceChannel)
        || current.mode === 'ssr' && !revealSSR(destination)) {
        event.preventDefault(); current.root.querySelector('[data-governance-route-status]').textContent = 'Existing sample destination unavailable; no output created.'; return;
      }
      current.focus = null; current.root.querySelector('[data-governance-route-status]').textContent = 'Existing sample view opened; no action, draft, message or receipt created.';
      if (current.mode === 'ssr' && link.dataset.governanceChannel) {
        if (location.href === link.href) event.preventDefault();
        pending = { key: current.key, channel: link.dataset.governanceChannel, oldURL: location.href, url: link.href, awaitingHash: location.href !== link.href, ownPop: location.href !== link.href };
      }
    }
  });
  document.addEventListener('toggle', event => { if (target(event)?.matches('[data-logic-detail]')) remember(); }, true);
  document.addEventListener('scroll', event => { if (target(event)?.matches('[data-governance-scroll]')) remember(); }, true);
  document.addEventListener('focusin', event => {
    const control = target(event)?.closest('[data-governance-focus]');
    if (control && current.available) current.focus = { node: control, token: control.dataset.governanceFocus, url: location.href };
    else if (current && event.target !== document.body) current.focus = null;
  });
  for (const name of ['pointerdown', 'keydown']) document.addEventListener(name, event => {
    if (!event.isTrusted || !(event.target instanceof Element)) return;
    pending = null; if (current?.focus && event.target !== current.focus.node) current.focus = null;
  }, true);
  window.addEventListener('blur', () => { if (current) current.focus = null; });
  window.addEventListener('hashchange', event => {
    if (current) current.focus = null;
    if (pending?.awaitingHash && event.oldURL === pending.oldURL && event.newURL === pending.url && location.href === pending.url) { pending.awaitingHash = false; pending.ownPop = false; } else pending = null;
  });
  window.addEventListener('popstate', () => { if (current) current.focus = null; if (pending?.awaitingHash && pending.ownPop && location.href === pending.url) pending.ownPop = false; else pending = null; });
  if (host) {
    const observer = new MutationObserver(() => { if (settling) return; settling = true; queueMicrotask(() => { settling = false; synchronize(); }); });
    observer.observe(host, { childList: true, subtree: true, characterData: true, attributes: true,
      attributeFilter: ['data-governance-source', 'data-governance-context', 'data-governance-label', 'data-governance-hub-source', 'data-governance-mode', 'data-governance-available', 'data-governance-source-available', 'data-governance-visible', 'data-governance-workflow-source', 'data-logic-record', 'data-logic-id', 'data-logic-category', 'data-logic-state', 'data-logic-search-text', 'data-risk-title', 'data-risk-detail', 'data-risk-owner', 'data-risk-score', 'data-gov-calendar-description'] });
    if (document.body) observer.observe(document.body, { childList: true, subtree: true, attributes: true, attributeFilter: ['id'] });
  }
  synchronize();
})();
"#;

#[cfg(test)]
mod tests {
    #[test]
    fn owner_loss_revokes_controls_and_defaults_without_replaying_intent() {
        crate::app::governance_owner_tests::check(super::SCRIPT);
    }
}
