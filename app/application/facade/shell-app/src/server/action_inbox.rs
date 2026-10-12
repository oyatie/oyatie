pub(super) const SCRIPT: &str = r#"
(() => {
  const host = document.getElementById('dashboard-island-root');
  let current = null, settling = false, pending = null;
  const owner = () => {
    const root = document.getElementById('command-center-workbench');
    return root?.dataset.inboxSource && host?.contains(root) ? root : null;
  };
  const rows = root => [...root.querySelectorAll('[data-inbox-row]')];
  const disclosures = root => [...root.querySelectorAll('[data-inbox-detail], [data-inbox-trace]')];
  const disclosureID = node => node.dataset.inboxDetail || node.dataset.inboxTrace;
  function records(root) {
    const result = new Map();
    try {
      for (const row of rows(root)) {
        const record = JSON.parse(row.dataset.inboxRecord), [meta, kind, ...fields] = record;
        if (record.length !== 6 || typeof meta?.id !== 'string' || !meta.id
          || typeof meta.owner !== 'string' || !meta.owner || result.has(meta.id)
          || !['task', 'approval'].includes(kind) || fields.some(value => typeof value !== 'string')
          || typeof meta.owned_by_viewer !== 'boolean' || typeof meta.blocking !== 'boolean'
          || meta.id !== row.dataset.inboxId || String(meta.owned_by_viewer) !== row.dataset.inboxOwned
          || String(meta.blocking) !== row.dataset.inboxBlocking) return null;
        result.set(meta.id, record);
      }
      return result;
    } catch { return null; }
  }
  function source(root) {
    return JSON.stringify([root.dataset.inboxSource, root.dataset.inboxContext, root.dataset.inboxLabel, root.dataset.inboxHubSource,
      rows(root).map(row => [row.dataset.inboxId, row.dataset.inboxRecord, row.dataset.inboxOwned, row.dataset.inboxBlocking, row.textContent]),
      [...root.querySelectorAll('[data-inbox-advice]')].map(node => node.dataset.inboxAdvice)]);
  }
  function remember() {
    if (!current?.root.isConnected) return;
    current.open = new Set(disclosures(current.root).filter(node => node.open).map(disclosureID));
    current.previews = new Set([...current.root.querySelectorAll('[data-inbox-packet]')].filter(node => node.open).map(node => node.dataset.inboxPacket));
  }
  function previews() {
    for (const preview of current.root.querySelectorAll('[data-inbox-packet]')) {
      const output = preview.querySelector('[data-inbox-preview-rows]');
      output.replaceChildren();
      for (const id of current.selected) {
        const [meta, kind, title, body, priority, requester] = current.records.get(id);
        const item = document.createElement('li'), heading = document.createElement('strong'), content = document.createElement('p'), detail = document.createElement('dl');
        heading.textContent = title; content.textContent = body;
        for (const [name, value] of [['Sample ID', meta.id], ['Kind', kind], ['Sample owner', meta.owner],
          ['Requester', requester], ['Sample priority', priority], ['Assigned to this sample role', String(meta.owned_by_viewer)],
          ['Sample blocker', String(meta.blocking)], ['Due, SLA and verified evidence', 'Unavailable']]) {
          const term = document.createElement('dt'), definition = document.createElement('dd');
          term.textContent = name; definition.textContent = value; detail.append(term, definition);
        }
        item.append(heading, content, detail); output.append(item);
      }
      preview.querySelector('[data-inbox-preview-empty]').hidden = current.selected.size !== 0;
    }
  }
  function apply() {
    const root = current.root, all = rows(root), visibleIDs = new Set();
    for (const row of all) {
      row.hidden = current.filter === 'mine' ? row.dataset.inboxOwned !== 'true'
        : current.filter === 'blocking' ? row.dataset.inboxBlocking !== 'true' : false;
      if (!row.hidden && current.records?.has(row.dataset.inboxId)) visibleIDs.add(row.dataset.inboxId);
    }
    current.selected = new Set([...current.selected].filter(id => visibleIDs.has(id)));
    for (const row of all) row.querySelector('[data-inbox-select]').checked = current.selected.has(row.dataset.inboxId);
    root.querySelectorAll('[data-inbox-filter]').forEach(button => {
      const active = button.dataset.inboxFilter === current.filter;
      button.setAttribute('aria-pressed', String(active)); button.classList.toggle('active', active);
    });
    root.querySelector('[data-inbox-selected-count]').textContent = current.selected.size;
    const selectAll = root.querySelector('[data-inbox-select-all]');
    selectAll.checked = visibleIDs.size > 0 && current.selected.size === visibleIDs.size;
    selectAll.indeterminate = current.selected.size > 0 && current.selected.size < visibleIDs.size;
    root.querySelector('[data-inbox-status]').textContent = `${visibleIDs.size} of ${all.length} sample rows shown · ${current.selected.size} selected`;
    root.querySelector('[data-inbox-empty]').hidden = visibleIDs.size !== 0;
    root.querySelector('[data-inbox-availability]').textContent = !current.records
      ? 'Sample record identity unavailable; local controls disabled. No live Inbox action is available.'
      : `${current.available ? 'Observed' : 'Retained'} anonymous sample: ${root.dataset.inboxLabel}; ${current.available ? 'local selection and previews only' : 'requested sample unavailable or pending; local controls disabled'}. Live Inbox actions unavailable.`;
    root.querySelectorAll('button, input').forEach(control => {
      control.disabled = !current.available || control.dataset.inboxUnavailable === 'true'
        || control.hasAttribute('data-inbox-preview') && current.selected.size === 0;
    });
    selectAll.disabled ||= visibleIDs.size === 0;
    root.querySelectorAll('summary, [data-inbox-route]').forEach(control => control.setAttribute('aria-disabled', String(!current.available)));
    root.querySelectorAll('[data-inbox-advice]').forEach(node => { node.hidden = current.dismissed.has(node.dataset.inboxAdvice); });
    previews();
  }
  function restoreFocus() {
    const saved = current.focus;
    if (!saved || saved.node.isConnected || !current.available) return;
    if (!document.hasFocus() || location.href !== saved.url) { current.focus = null; return; }
    const active = document.activeElement;
    if (active && active !== document.body && active.isConnected) { current.focus = null; return; }
    const target = [...current.root.querySelectorAll('[data-inbox-focus]')]
      .find(node => node.dataset.inboxFocus === saved.token && !node.disabled && node.getClientRects().length);
    if (!target) { current.focus = null; return; }
    current.focus = null; target.focus({ preventScroll: true });
    if (document.activeElement === target) current.focus = { ...saved, node: target };
  }
  function synchronize() {
    const root = owner(); if (!root) return;
    const key = source(root), registry = records(root), available = root.dataset.inboxAvailable === 'true' && !!registry, mode = root.dataset.inboxMode;
    if (current?.root === root && current.key === key && current.available === available && current.mode === mode) return;
    remember(); const previous = current, same = previous?.key === key;
    if (previous && !same) pending = null;
    current = { root, key, records: registry, available, mode, filter: same ? previous.filter : 'all',
      selected: same ? previous.selected : new Set(), open: same ? previous.open : new Set(),
      previews: same ? previous.previews : new Set(), dismissed: same ? previous.dismissed : new Set(), focus: same ? previous.focus : null };
    disclosures(root).forEach(node => { node.open = current.open.has(disclosureID(node)); });
    root.querySelectorAll('[data-inbox-packet]').forEach(node => { node.open = current.previews.has(node.dataset.inboxPacket); });
    root.querySelector('[data-inbox-reset]').hidden = same || !previous;
    apply(); restoreFocus();
    if (pending && current.mode === 'island' && current.available) {
      const request = pending; pending = null;
      if (request.key === key && location.href === request.url) openChannel(request.channel, request.itemID);
    }
  }
  function target(event) {
    if (!(event.target instanceof Element) || !event.target.isConnected) return null;
    const root = event.target.closest('#command-center-workbench[data-inbox-source]');
    if (!root || root !== owner()) return null;
    synchronize(); return current?.root === root ? event.target : null;
  }
  function workspace() {
    const hub = document.getElementById('work-hub'), root = hub?.querySelector('.communications-hub[data-hub-source]');
    return root && host.contains(hub) && root.dataset.hubContext === current.root.dataset.inboxContext
      && root.dataset.hubSource === current.root.dataset.inboxHubSource ? root : null;
  }
  function channelReady(root, channel) {
    return root?.querySelector(`#hub-tab-${channel.toLowerCase()}`)?.getAttribute('aria-selected') === 'true'
      && root.querySelector('[data-comms-product-board]')?.dataset.commsBoardSurface === channel
      && root.querySelector('.hub-list') && root.querySelector('.hub-detail') && root.querySelector('.hub-composer textarea');
  }
  function openChannel(channel, itemID) {
    if (!['Mail', 'Community'].includes(channel) || !current.available) return false;
    let root = workspace(); if (!root || root.dataset.hubMode !== current.mode) return false;
    if (channelReady(root, channel)) return true;
    if (current.mode === 'island') {
      const tab = root.querySelector(`#hub-tab-${channel.toLowerCase()}`);
      if (!tab || tab.disabled) return false; tab.click();
    } else {
      const template = [...current.root.querySelectorAll('[data-inbox-hub-template]')].find(node => node.dataset.inboxHubTemplate === channel);
      const replacement = template?.content.firstElementChild?.cloneNode(true);
      if (!replacement || replacement.dataset.hubContext !== current.root.dataset.inboxContext
        || replacement.dataset.hubSource !== current.root.dataset.inboxHubSource || !channelReady(replacement, channel)) return false;
      root.replaceWith(replacement);
    }
    const key = current.key;
    requestAnimationFrame(() => {
      if (current?.key !== key || !current.available) return;
      current.root.querySelector('[data-inbox-route-status]').textContent = channelReady(workspace(), channel)
        ? `${channel} existing sample opened${itemID ? ` for sample ${itemID}` : ''}; no draft, message, publication or receipt created.` : `${channel} sample unavailable; no output created.`;
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
    if (document.hasFocus()) {
      if (!destination.hasAttribute('tabindex')) destination.setAttribute('tabindex', '-1');
      destination.focus({ preventScroll: true }); destination.scrollIntoView({ block: 'start' });
    }
    return true;
  }
  document.addEventListener('change', event => {
    const control = target(event); if (!control) return;
    if (!current.available) { apply(); return; }
    if (control.matches('[data-inbox-select-all]')) current.selected = new Set(control.checked ? rows(current.root).filter(row => !row.hidden).map(row => row.dataset.inboxId) : []);
    else if (control.matches('[data-inbox-select]')) {
      const row = control.closest('[data-inbox-row]'); if (!row || row.hidden || !current.records.has(row.dataset.inboxId)) return;
      if (control.checked) current.selected.add(row.dataset.inboxId); else current.selected.delete(row.dataset.inboxId);
    } else return;
    apply();
  });
  document.addEventListener('click', event => {
    const control = target(event); if (!control) return;
    if (!current.available) { event.preventDefault(); return; }
    const filter = control.closest('[data-inbox-filter]'), preview = control.closest('[data-inbox-preview]'), close = control.closest('[data-inbox-close]');
    if (filter && ['all', 'mine', 'blocking'].includes(filter.dataset.inboxFilter)) { current.filter = filter.dataset.inboxFilter; apply(); }
    else if (preview && current.selected.size) {
      current.root.querySelector(`[data-inbox-packet="${preview.dataset.inboxPreview}"]`).open = true; remember();
    } else if (close) {
      const detail = close.closest('details'); detail.open = false; remember();
      if (document.hasFocus()) detail.querySelector('summary').focus({ preventScroll: true });
    } else if (control.closest('[data-inbox-dismiss]')) {
      current.dismissed.add(control.closest('[data-inbox-advice]').dataset.inboxAdvice); apply();
      if (document.hasFocus()) current.root.querySelector('#copilot-workbench-title').focus({ preventScroll: true });
    } else {
      const link = control.closest('[data-inbox-route]'); if (!link) return;
      const itemID = link.closest('[data-inbox-row]')?.dataset.inboxId;
      const destination = document.getElementById(link.getAttribute('href').slice(1));
      if (!destination || link.dataset.inboxChannel && !openChannel(link.dataset.inboxChannel, itemID)
        || current.mode === 'ssr' && !revealSSR(destination)) {
        event.preventDefault(); current.root.querySelector('[data-inbox-route-status]').textContent = 'Existing sample destination unavailable; no output created.'; return;
      }
      current.focus = null;
      current.root.querySelector('[data-inbox-route-status]').textContent = `Existing sample view opened${itemID ? ` for sample ${itemID}` : ''}; no action, draft or receipt created.`;
      if (current.mode === 'ssr' && link.dataset.inboxChannel) {
        if (location.href === link.href) event.preventDefault();
        pending = { key: current.key, channel: link.dataset.inboxChannel, itemID, oldURL: location.href, url: link.href,
          awaitingHash: location.href !== link.href, ownPop: location.href !== link.href };
      }
    }
  });
  document.addEventListener('toggle', event => { if (target(event)?.matches('[data-inbox-detail], [data-inbox-trace], [data-inbox-packet]')) remember(); }, true);
  document.addEventListener('focusin', event => {
    const control = target(event)?.closest('[data-inbox-focus]');
    if (control && current.available) current.focus = { node: control, token: control.dataset.inboxFocus, url: location.href };
    else if (current && event.target !== document.body) current.focus = null;
  });
  for (const name of ['pointerdown', 'keydown']) document.addEventListener(name, event => {
    if (!event.isTrusted || !(event.target instanceof Element)) return;
    pending = null; if (current?.focus && event.target !== current.focus.node) current.focus = null;
  }, true);
  document.addEventListener('keydown', event => { if (target(event) && !current.available && ['Enter', ' ', 'Spacebar'].includes(event.key)) event.preventDefault(); }, true);
  window.addEventListener('blur', () => { if (current) current.focus = null; });
  window.addEventListener('hashchange', event => {
    if (current) current.focus = null;
    if (pending?.awaitingHash && event.oldURL === pending.oldURL && event.newURL === pending.url && location.href === pending.url) {
      pending.awaitingHash = false; pending.ownPop = false;
    } else pending = null;
  });
  window.addEventListener('popstate', () => {
    if (current) current.focus = null;
    if (pending?.awaitingHash && pending.ownPop && location.href === pending.url) pending.ownPop = false; else pending = null;
  });
  if (host) new MutationObserver(() => {
    if (settling) return; settling = true; queueMicrotask(() => { settling = false; synchronize(); });
  }).observe(host, { childList: true, subtree: true, characterData: true, attributes: true,
    attributeFilter: ['data-inbox-source', 'data-inbox-context', 'data-inbox-label', 'data-inbox-hub-source', 'data-inbox-mode', 'data-inbox-available',
      'data-inbox-id', 'data-inbox-record', 'data-inbox-owned', 'data-inbox-blocking'] });
  synchronize();
})();
"#;
