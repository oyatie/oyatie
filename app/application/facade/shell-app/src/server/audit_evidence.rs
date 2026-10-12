pub(super) const SCRIPT: &str = r##"
(() => {
  const host = document.getElementById('dashboard-island-root');
  let current = null, pending = null, settling = false, restoring = false;
  const owners = () => {
    const evidence = document.getElementById('evidence-spine'), audit = document.getElementById('resource-audit-console');
    return evidence && audit && host?.contains(evidence) && host.contains(audit) ? [evidence, audit] : null;
  };
  const controls = roots => roots.flatMap(root => [...root.querySelectorAll('[data-ae-select]')]);
  const links = roots => roots.flatMap(root => [...root.querySelectorAll('a[data-ae-route]')]);
  const normalize = value => String(value).normalize('NFC').trim().toLocaleLowerCase();
  const fields = {
    event: ['State', 'Receipt', 'Title', 'Body', 'Source', 'Owner', 'SLA'],
    node: ['Key', 'Label', 'Value', 'Description'],
    audit: ['Time', 'Event', 'Actor', 'Receipt', 'State'],
    source: ['Key', 'Title', 'Receipt', 'Route', 'Owner', 'State']
  };
  function facts(roots) {
    return JSON.stringify([controls(roots).map(node => [node.dataset.aeSelect, node.dataset.aeRecord, node.dataset.aeFocus,
      [...(node.closest('[data-ae-record-row]') || node).querySelectorAll('[data-ae-value]')].map(value => [value.dataset.aeValue, value.textContent]),
      node.dataset.evidenceState, node.closest('[data-evidence-event]')?.dataset.evidenceState,
      node.dataset.objectNode, node.dataset.objectLabel, node.dataset.receiptSource, node.dataset.receiptTitle,
      node.dataset.receiptId, node.dataset.receiptRoute, node.dataset.receiptOwner, node.dataset.receiptState]),
      links(roots).map(node => [node.dataset.aeRouteKey, node.dataset.aeRoute, node.getAttribute('href'), node.dataset.aeChannel || null, node.dataset.aeFocus]),
      [...roots[0].querySelectorAll('[data-ae-workflow]')].map(node => [node.dataset.aeWorkflow, node.textContent]),
      roots.flatMap(root => [...root.querySelectorAll('[data-ae-unavailable]')].map(node => [node.dataset.aeEffect, node.getAttribute('title'), node.textContent]))]);
  }
  function valid(roots) {
    try {
      const [root, audit] = roots, source = JSON.parse(root.dataset.aeSource);
      if (!Array.isArray(source) || source.length !== 7 || !['tenant-admin', 'corporate-office', 'healthcare-clinician'].includes(source[0])
        || source.slice(0, 5).some(value => typeof value !== 'string' || !value)
        || source[0] !== root.dataset.aeContext || source[1] !== root.dataset.aeLabel || source[2] !== root.dataset.aeHubSource
        || !['ssr', 'island'].includes(root.dataset.aeMode)
        || ['aeSource', 'aeContext', 'aeLabel', 'aeHubSource', 'aeMode'].some(name => audit.dataset[name] !== root.dataset[name])
        || roots.some(owner => !['true', 'false'].includes(owner.dataset.aeSourceAvailable) || !['true', 'false'].includes(owner.dataset.aeAvailable)
          || !owner.querySelector('[data-ae-inspection]') || !owner.querySelector('[data-ae-inspect-title]') || owner.querySelectorAll('[data-ae-inspect-field]').length !== 7
          || [...owner.querySelectorAll('[data-ae-inspect-field]')].some((slot, index) => slot.dataset.aeInspectField !== String(index) || !slot.querySelector('[data-ae-inspect-label]') || !slot.querySelector('[data-ae-inspect-value]'))
          || !owner.querySelector('[data-ae-close]') || !owner.querySelector('[data-ae-availability]') || !owner.querySelector('[data-ae-reset]')
          || !owner.querySelector('[data-ae-route-status]') || !owner.querySelector('[data-ae-unavailable-reason]') || !owner.querySelector('[data-ae-fixture]'))
        || !Array.isArray(source[5]) || source[5].length !== 23 || !Array.isArray(source[6])) return false;
      const records = new Map(), counts = {event: 0, node: 0, audit: 0, source: 0};
      for (const record of source[5]) {
        if (!Array.isArray(record) || record.length !== 2 || typeof record[0] !== 'string' || records.has(record[0])) return false;
        const family = record[0].split(':')[0], names = fields[family], pairs = record[1];
        if (!names || !record[0].split(':')[1] || !Array.isArray(pairs) || pairs.length !== names.length
          || pairs.some((pair, index) => !Array.isArray(pair) || pair.length !== 2 || pair[0] !== names[index] || typeof pair[1] !== 'string' || !pair[1])) return false;
        records.set(record[0], pairs); counts[family]++;
      }
      if (JSON.stringify(counts) !== JSON.stringify({event: 6, node: 8, audit: 5, source: 4})) return false;
      const seen = new Set();
      for (const node of controls(roots)) {
        const token = node.dataset.aeSelect, pairs = records.get(token), family = token?.split(':')[0];
        if (!pairs || seen.has(token) || node.dataset.aeFocus !== token || JSON.stringify(JSON.parse(node.dataset.aeRecord)) !== JSON.stringify(pairs)
          || (['event', 'node'].includes(family) ? !root.contains(node) : !audit.querySelector('#resource-panel-audit')?.contains(node))) return false;
        const values = new Map(pairs), visible = [...(node.closest('[data-ae-record-row]') || node).querySelectorAll('[data-ae-value]')], valueNames = new Set();
        if (!visible.length || visible.some(value => valueNames.has(value.dataset.aeValue) || !valueNames.add(value.dataset.aeValue)
          || value.textContent !== values.get(value.dataset.aeValue))) return false;
        const required = family === 'node' ? ['Label', 'Value'] : family === 'source' ? ['Title', 'Receipt'] : fields[family];
        if (required.some(name => !valueNames.has(name))) return false;
        if (family === 'event' && (token !== `event:${values.get('Receipt')}` || node.closest('[data-evidence-event]')?.dataset.evidenceState !== values.get('State'))
          || family === 'node' && (token !== `node:${values.get('Key')}` || node.dataset.objectNode !== values.get('Key') || node.dataset.objectLabel !== values.get('Label'))
          || family === 'audit' && token !== `audit:${values.get('Receipt')}`
          || family === 'source' && (token !== `source:${values.get('Key')}` || ['Key', 'Title', 'Receipt', 'Route', 'Owner', 'State'].some((name, index) => node.dataset[['receiptSource', 'receiptTitle', 'receiptId', 'receiptRoute', 'receiptOwner', 'receiptState'][index]] !== values.get(name)))) return false;
        seen.add(token);
      }
      if (seen.size !== records.size) return false;
      const routes = new Map();
      for (const route of source[6]) {
        if (!Array.isArray(route) || route.length !== 3 || typeof route[0] !== 'string' || !route[0] || routes.has(route[0])
          || typeof route[1] !== 'string' || !route[1] || route[2] !== null && !['Mail', 'Community', 'Messenger'].includes(route[2])) return false;
        routes.set(route[0], route);
      }
      const routed = new Set();
      for (const link of links(roots)) {
        const route = routes.get(link.dataset.aeRouteKey);
        if (!route || link.dataset.aeRoute !== route[1] || link.getAttribute('href') !== `#${route[1]}`
          || (link.dataset.aeChannel || null) !== route[2] || link.dataset.aeFocus !== route[0]) return false;
        if (routed.has(route[0])) return false; routed.add(route[0]);
      }
      if (routed.size !== routes.size || routes.size !== (source[0] === 'healthcare-clinician' ? 38 : 40)) return false;
      const effects = ['evidence:run-review', 'evidence:export', 'evidence:attach', 'trust:seal-receipt', 'trust:trace-lineage', 'audit:seal-packet', 'stitch:seal'];
      if (source[0] === 'healthcare-clinician') effects.push('trust:route-finops', 'trust:route-finops:graph');
      const unavailable = roots.flatMap(owner => [...owner.querySelectorAll('[data-ae-unavailable]')]);
      if (JSON.stringify(unavailable.map(node => node.dataset.aeEffect).sort()) !== JSON.stringify(effects.sort()) || unavailable.some(node => node.dataset.aeUnavailable !== 'true' || !node.getAttribute('title') || !node.textContent)) return false;
      const workflow = [...root.querySelectorAll('[data-ae-workflow]')];
      if (workflow.length !== 3 || workflow.some(node => node.textContent !== source[node.dataset.aeWorkflow === 'name' ? 3 : node.dataset.aeWorkflow === 'goal' ? 4 : 7])) return false;
      if (!root.querySelector('[data-evidence-search]') || !root.querySelector('[data-ae-clear]') || !root.querySelector('[data-ae-empty]')
        || !root.querySelector('[data-evidence-status]') || JSON.stringify([...root.querySelectorAll('[data-evidence-filter]')].map(node => node.dataset.evidenceFilter)) !== JSON.stringify(['all', 'blocking', 'review', 'sealed', 'watch'])) return false;
      return true;
    } catch { return false; }
  }
  function inViewport(node) {
    const box = node.getBoundingClientRect();
    return box.width > 0 && box.height > 0 && box.bottom > 0 && box.top < innerHeight && box.right > 0 && box.left < innerWidth;
  }
  function toolbar() {
    const root = current.roots[1], active = root.querySelector('#resource-tab-audit')?.getAttribute('aria-selected') === 'true';
    const nodes = [...root.querySelectorAll('[data-resource-search], [data-resource-filter], [data-resource-action]')], status = root.querySelector('[data-resource-status]');
    if (active && nodes.length === 7 && status) {
      if (!current.toolbar || current.toolbar.root !== root) current.toolbar = { root, status, text: status.textContent, role: status.getAttribute('role'),
        nodes: nodes.map(node => ({ node, disabled: node.disabled, title: node.getAttribute('title'), aria: node.getAttribute('aria-disabled'), value: node.value })) };
      nodes.forEach(node => { node.disabled = true; const reason = 'Live Audit query, filters, refresh and export are unavailable; no output created.'; if (node.title !== reason) node.title = reason; node.setAttribute('aria-disabled', 'true'); });
      if (status.textContent !== 'Audit toolbar unavailable · no live query, refresh or export; inspect sample rows below.') status.textContent = 'Audit toolbar unavailable · no live query, refresh or export; inspect sample rows below.';
      status.setAttribute('role', 'status');
    } else if (current.toolbar) {
      const saved = current.toolbar; current.toolbar = null;
      if (saved.root === root) {
        saved.nodes.forEach(({node, disabled, title, aria, value}) => { node.disabled = disabled; if (title === null) node.removeAttribute('title'); else node.setAttribute('title', title); if (aria === null) node.removeAttribute('aria-disabled'); else node.setAttribute('aria-disabled', aria); if (node.matches('[data-resource-search]')) node.value = value; });
        saved.status.textContent = saved.text; if (saved.role === null) saved.status.removeAttribute('role'); else saved.status.setAttribute('role', saved.role);
      }
    }
    if (!current.restoreTab) current.tab = root.querySelector('.resource-tabs [role="tab"][aria-selected="true"]')?.dataset.resourceTab;
  }
  function apply() {
    const [root] = current.roots; let count = 0;
    for (const node of controls(current.roots)) {
      const pairs = current.records.get(node.dataset.aeSelect), row = node.closest('[data-evidence-event]');
      if (row) {
        const values = new Map(pairs || []);
        row.hidden = !(current.filter === 'all' || current.filter === values.get('State')) || !normalize((pairs || []).map(pair => pair[1]).join(' ')).includes(normalize(current.query));
        if (!row.hidden) count++; else if (current.selected === node.dataset.aeSelect) current.selected = null;
      }
      const selected = current.available && current.selected === node.dataset.aeSelect;
      node.disabled = !current.available; node.setAttribute('aria-pressed', String(selected)); node.classList.toggle('active', selected);
    }
    const search = root.querySelector('[data-evidence-search]'); if (search) { search.value = current.query; search.disabled = !current.available; }
    root.querySelectorAll('[data-evidence-filter]').forEach(node => { const selected = node.dataset.evidenceFilter === current.filter; node.disabled = !current.available; node.setAttribute('aria-pressed', String(selected)); node.classList.toggle('active', selected); });
    const clear = root.querySelector('[data-ae-clear]'); if (clear) clear.disabled = !current.available;
    const status = root.querySelector('[data-evidence-status]'); if (status) status.textContent = `${count} of 6 sample events · ${current.filter} states${current.selected ? ` · selected ${current.selected}` : ''}`;
    const empty = root.querySelector('[data-ae-empty]'); if (empty) empty.hidden = count !== 0;
    for (const owner of current.roots) {
      const notice = owner.querySelector('[data-ae-availability]'); if (notice) notice.textContent = `${current.available ? 'Observed' : 'Retained'} anonymous sample: ${owner.dataset.aeLabel}. ${current.available ? 'Local illustrative inspection only' : 'Requested sample or native source unavailable; local controls disabled'}. No tenant or role verified; live Audit readiness unknown.`;
      owner.querySelectorAll('[data-ae-unavailable]').forEach(node => { node.disabled = true; });
      owner.querySelectorAll('[data-ae-route]').forEach(node => node.setAttribute('aria-disabled', String(!current.available)));
      const inspection = owner.querySelector('[data-ae-inspection]'), selected = controls([owner]).find(node => node.dataset.aeSelect === current.selected);
      if (inspection) {
        inspection.hidden = !current.available || !selected;
        if (selected && current.available) {
          const pairs = current.records.get(current.selected), title = inspection.querySelector('[data-ae-inspect-title]');
          if (title) title.textContent = `Illustrative ${current.selected}`;
          inspection.querySelectorAll('[data-ae-inspect-field]').forEach((slot, index) => { const pair = pairs[index]; slot.hidden = !pair; if (pair) { slot.querySelector('[data-ae-inspect-label]').textContent = pair[0]; slot.querySelector('[data-ae-inspect-value]').textContent = pair[1]; } });
        }
      }
      owner.querySelectorAll('[data-ae-close]').forEach(node => { node.disabled = !current.available; });
    }
    toolbar();
  }
  function restoreFocus() {
    const saved = current.focus;
    if (!saved || saved.node.isConnected || !current.available || current.restoreTab) return;
    if (!document.hasFocus() || location.href !== saved.url) { current.focus = null; return; }
    const active = document.activeElement;
    if (active && active !== document.body && active.isConnected && active !== current.restoredTabFocus) { current.focus = null; return; }
    const owner = current.roots.find(root => root.id === saved.owner), target = [...(owner?.querySelectorAll('[data-ae-focus]') || [])].find(node => node.dataset.aeFocus === saved.token && !node.disabled && node.getClientRects().length);
    current.focus = null; current.restoredTabFocus = null;
    if (target) { target.focus({preventScroll: true}); if (document.activeElement === target) { current.focus = {...saved, node: target}; if (saved.visible && !inViewport(target)) target.scrollIntoView({block: 'center', inline: 'nearest'}); current.focus.visible = inViewport(target); } }
  }
  function restoreTab() {
    if (restoring || !current.restoreTab || !current.available || !document.hasFocus()) return;
    const tab = current.roots[1].querySelector(`#resource-tab-${current.restoreTab}`), saved = current.focus, active = document.activeElement;
    if (!tab || tab.disabled) return;
    restoring = true; if (tab.getAttribute('aria-selected') !== 'true') tab.click(); restoring = false; current.focus = saved;
    if (active?.isConnected && active !== document.body && active !== tab && document.activeElement === tab) active.focus({preventScroll: true});
    else if (saved && !saved.node.isConnected && document.activeElement === tab) current.restoredTabFocus = tab;
    if (tab.getAttribute('aria-selected') === 'true') { current.tab = current.restoreTab; current.restoreTab = null; }
  }
  function synchronize() {
    const roots = owners(); if (!roots) { if (current?.available) { current.available = false; apply(); } return; }
    const key = JSON.stringify([roots.map(root => [root.dataset.aeSource, root.dataset.aeContext, root.dataset.aeLabel, root.dataset.aeHubSource]), facts(roots)]);
    const verified = valid(roots), available = verified && roots.every(root => root.dataset.aeSourceAvailable === 'true' && root.dataset.aeAvailable === 'true'), mode = roots[0].dataset.aeMode;
    if (current?.roots[0] === roots[0] && current.roots[1] === roots[1] && current.key === key && current.available === available && current.verified === verified && current.mode === mode) { restoreTab(); toolbar(); restoreFocus(); resumeChannel(); return; }
    const previous = current, same = previous?.key === key && verified && previous.verified;
    if (!same || !available) pending = null;
    let records = []; try { const source = JSON.parse(roots[0].dataset.aeSource); if (verified) records = source[5]; } catch {}
    current = { roots, key, available, verified, mode, records: new Map(records), query: same ? previous.query : '', filter: same ? previous.filter : 'all', selected: same ? previous.selected : null,
      focus: same ? previous.focus : null, tab: same ? previous.tab : null, restoreTab: same && previous.roots[1] !== roots[1] ? previous.tab : same ? previous.restoreTab : null,
      toolbar: previous?.roots[1] === roots[1] ? previous.toolbar : null };
    for (const root of roots) { const reset = root.querySelector('[data-ae-reset]'); if (reset) reset.hidden = same || !previous; }
    restoreTab(); apply(); restoreFocus();
    if (current.restoreTab) { const bound = current; requestAnimationFrame(() => { if (current === bound) { restoreTab(); toolbar(); restoreFocus(); } }); }
    resumeChannel();
  }
  function target(event) {
    if (!(event.target instanceof Element) || !event.target.isConnected) return null;
    const roots = owners(), root = event.target.closest('#evidence-spine, #resource-audit-console');
    if ((!roots?.includes(root) && !(current?.roots.includes(root) && host?.contains(root))) || event.target.closest('#payroll-evidence-preview')) return null;
    synchronize(); return current?.roots.includes(root) ? event.target : null;
  }
  function workspace() {
    const hub = document.getElementById('work-hub'), root = hub?.querySelector('.communications-hub[data-hub-source]'), owner = current.roots[0];
    return root && host.contains(hub) && root.dataset.hubContext === owner.dataset.aeContext && root.dataset.hubSource === owner.dataset.aeHubSource && root.dataset.hubMode === current.mode ? root : null;
  }
  function channelReady(root, channel) {
    const tab = root?.querySelector(`#hub-tab-${channel.toLowerCase()}`), composer = root?.querySelector('.hub-composer textarea');
    return tab?.getAttribute('aria-selected') === 'true' && (current.mode === 'ssr' || !tab.disabled && composer && !composer.disabled)
      && root.querySelector('[data-comms-product-board]')?.dataset.commsBoardSurface === channel && root.querySelector('.hub-list') && root.querySelector('.hub-detail') && composer;
  }
  function resumeChannel() {
    const request = pending; if (!request) return;
    if (request.key !== current.key || !current.available) { pending = null; return; }
    if (location.href !== request.url) { if (request.awaitingHash && location.href === request.oldURL) return; pending = null; return; }
    if (current.mode !== 'island') return;
    const root = workspace(), tab = root?.querySelector(`#hub-tab-${request.channel.toLowerCase()}`), composer = root?.querySelector('.hub-composer textarea');
    if (!tab || tab.disabled || !composer || composer.disabled) return;
    if (channelReady(root, request.channel)) { pending = null; return; }
    if (request.attempted === tab) return;
    request.attempted = tab; openChannel(request.channel);
    if (pending === request && channelReady(workspace(), request.channel)) pending = null;
  }
  function openChannel(channel) {
    if (!['Mail', 'Community', 'Messenger'].includes(channel) || !current.available) return false;
    const root = workspace(); if (!root) return false;
    if (channelReady(root, channel)) return true;
    if (current.mode === 'island') { const tab = root.querySelector(`#hub-tab-${channel.toLowerCase()}`); if (!tab || tab.disabled) return false; tab.click(); }
    else {
      const owner = current.roots[0], template = [...owner.querySelectorAll('[data-ae-hub-template]')].find(node => node.dataset.aeHubTemplate === channel), replacement = template?.content.firstElementChild?.cloneNode(true);
      if (!replacement || replacement.dataset.hubContext !== owner.dataset.aeContext || replacement.dataset.hubSource !== owner.dataset.aeHubSource || replacement.dataset.hubMode !== current.mode || !channelReady(replacement, channel)) return false;
      root.replaceWith(replacement);
    }
    const key = current.key;
    requestAnimationFrame(() => { if (current?.key === key && current.available) for (const owner of current.roots) owner.querySelector('[data-ae-route-status]').textContent = channelReady(workspace(), channel) ? `${channel} existing sample opened; no draft, message or receipt created.` : `${channel} sample unavailable; no output created.`; });
    return true;
  }
  function revealSSR(destination) {
    const panel = destination.closest('[role="tabpanel"]');
    if (panel) {
      const tab = document.getElementById(panel.getAttribute('aria-labelledby')), group = tab?.closest('[role="tablist"]');
      if (!tab || tab.disabled || tab.getAttribute('aria-controls') !== panel.id || tab.parentElement !== group || group.hasAttribute('data-native-hub-tabs') || group.classList.contains('hub-tabs')) return false;
      tab.click(); if (tab.getAttribute('aria-selected') !== 'true' || !panel.classList.contains('active')) return false;
    }
    if (document.hasFocus()) { if (!destination.hasAttribute('tabindex')) destination.setAttribute('tabindex', '-1'); destination.focus({preventScroll: true}); destination.scrollIntoView({block: 'start'}); }
    return true;
  }
  function closeInspection() {
    const selected = current.selected, node = controls(current.roots).find(control => control.dataset.aeSelect === selected);
    current.selected = null; apply();
    if (document.hasFocus() && node && !node.disabled && node.getClientRects().length) node.focus({preventScroll: true});
  }
  document.addEventListener('input', event => { const input = target(event)?.closest('[data-evidence-search]'); if (!input) return; if (current.available && !input.disabled) { current.query = input.value; apply(); } else input.value = current.query; });
  document.addEventListener('click', event => {
    const control = target(event); if (!control || event.defaultPrevented || event.button !== 0 || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    const action = control.closest('[data-ae-select], [data-evidence-filter], [data-ae-clear], [data-ae-close], [data-ae-route], [data-ae-unavailable]');
    if (!action) return;
    if (!current.available || action.matches('[data-ae-unavailable]')) { event.preventDefault(); return; }
    if (action.matches('[data-ae-select]')) { current.selected = action.dataset.aeSelect; apply(); const owner = action.closest('#evidence-spine, #resource-audit-console'); owner.querySelector('[data-ae-route-status]').textContent = `Inspecting illustrative ${current.selected}; no live verification or operation.`; }
    else if (action.matches('[data-evidence-filter]')) { current.filter = action.dataset.evidenceFilter; apply(); }
    else if (action.matches('[data-ae-clear]')) { current.query = ''; current.filter = 'all'; apply(); }
    else if (action.matches('[data-ae-close]')) closeInspection();
    else {
      const destination = document.getElementById(action.dataset.aeRoute);
      if (!destination || !host.contains(destination) || action.dataset.aeChannel && !openChannel(action.dataset.aeChannel) || current.mode === 'ssr' && !revealSSR(destination)) {
        event.preventDefault(); action.closest('#evidence-spine, #resource-audit-console').querySelector('[data-ae-route-status]').textContent = 'Existing sample destination unavailable; no output created.'; return;
      }
      current.focus = null; action.closest('#evidence-spine, #resource-audit-console').querySelector('[data-ae-route-status]').textContent = 'Existing sample view opened; no action, draft, message or receipt created.';
      if (current.mode === 'ssr' && action.dataset.aeChannel) { if (location.href === action.href) event.preventDefault(); pending = {key: current.key, channel: action.dataset.aeChannel, oldURL: location.href, url: action.href, awaitingHash: location.href !== action.href, ownPop: location.href !== action.href}; }
    }
  }, true);
  document.addEventListener('focusin', event => {
    if (restoring) return;
    const control = target(event)?.closest('[data-ae-focus]');
    if (control && current.available) current.focus = {node: control, owner: control.closest('#evidence-spine, #resource-audit-console').id, token: control.dataset.aeFocus, url: location.href, visible: inViewport(control)};
    else if (current && event.target !== document.body) current.focus = null;
  });
  document.addEventListener('scroll', () => { const saved = current?.focus; if (saved?.node.isConnected && document.activeElement === saved.node) saved.visible = inViewport(saved.node); }, true);
  for (const name of ['pointerdown', 'keydown']) document.addEventListener(name, event => {
    if (!event.isTrusted || !(event.target instanceof Element)) return;
    pending = null; if (current && event.target.closest('.resource-tabs')) current.restoreTab = null; if (current?.focus && event.target !== current.focus.node) current.focus = null;
  }, true);
  document.addEventListener('keydown', event => {
    const control = target(event); if (!control) return;
    if (event.key === 'Escape' && current.selected && current.available) { event.preventDefault(); closeInspection(); }
    else if (!current.available && ['Enter', ' ', 'Spacebar'].includes(event.key) && control.closest('[data-ae-select], [data-evidence-filter], [data-ae-clear], [data-ae-close], [data-ae-route]')) event.preventDefault();
  }, true);
  window.addEventListener('blur', () => { if (current) { current.focus = null; current.restoreTab = null; } pending = null; });
  window.addEventListener('hashchange', event => { if (current) current.focus = null; if (pending?.awaitingHash && event.oldURL === pending.oldURL && event.newURL === pending.url && location.href === pending.url) { pending.awaitingHash = false; pending.ownPop = false; synchronize(); } else pending = null; });
  window.addEventListener('popstate', () => { if (current) current.focus = null; if (pending?.awaitingHash && pending.ownPop && location.href === pending.url) pending.ownPop = false; else pending = null; });
  if (host) new MutationObserver(records => { if (records.every(record => record.type === 'attributes' && record.attributeName === 'disabled' && !record.target.closest('.communications-hub[data-hub-source]')) || settling) return; settling = true; queueMicrotask(() => { settling = false; synchronize(); }); })
    .observe(host, {childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ['data-ae-record-row', 'data-ae-inspection', 'data-ae-inspect-field', 'data-ae-inspect-title', 'data-ae-inspect-label', 'data-ae-inspect-value', 'data-ae-close', 'data-ae-availability', 'data-ae-reset', 'data-ae-route-status', 'data-ae-unavailable-reason', 'data-ae-fixture', 'data-ae-unavailable', 'data-ae-effect', 'data-ae-clear', 'data-ae-empty', 'data-evidence-search', 'data-evidence-filter', 'data-evidence-event', 'data-evidence-status', 'title', 'data-ae-source', 'data-ae-context', 'data-ae-label', 'data-ae-hub-source', 'data-ae-source-available', 'data-ae-available', 'data-ae-mode', 'data-ae-select', 'data-ae-record', 'data-ae-focus', 'data-ae-value', 'data-ae-workflow', 'data-ae-route', 'data-ae-route-key', 'data-ae-channel', 'data-evidence-state', 'data-object-node', 'data-object-label', 'data-receipt-source', 'data-receipt-title', 'data-receipt-id', 'data-receipt-route', 'data-receipt-owner', 'data-receipt-state', 'href', 'aria-selected', 'data-hub-source', 'data-hub-context', 'data-hub-mode', 'data-comms-board-surface', 'disabled']});
  synchronize();
})();
"##;

#[cfg(test)]
#[path = "audit_evidence_test.rs"]
mod tests;
