pub(super) const SCRIPT: &str = r##"
(() => {
  const host = document.getElementById('dashboard-island-root');
  if (!host) return;
  let current = null, queued = false, restoring = false;
  const all = (root, selector) => [...root.querySelectorAll(selector)];
  const attr = (node, name, value) => { if (node.getAttribute(name) !== String(value)) node.setAttribute(name, String(value)); };
  const text = (node, value) => { if (node && node.textContent !== String(value)) node.textContent = String(value); };
  const disabled = (node, value) => { if (node.disabled !== value) node.disabled = value; };
  const normalize = value => String(value).normalize('NFC').trim().toLocaleLowerCase();
  const names = {region: ['Region', 'State', 'Capacity', 'Description'], workload: ['Key', 'Title', 'Service', 'Cell', 'State', 'Route', 'Receipt', 'Description'],
    resource: ['Key', 'Kind', 'Name', 'Region', 'Owner', 'State', 'Monthly', 'Risk', 'Description'], gate: ['Label', 'Detail', 'State', 'Progress']};
  const visible = {region: ['Region', 'State', 'Capacity'], workload: ['Title', 'Service', 'Cell', 'State'], resource: ['Kind', 'Name', 'Region', 'Owner', 'State', 'Monthly', 'Description'], gate: ['Label', 'Progress']};
  const owners = () => {
    const topology = document.getElementById('cockpit-panel-topology'), resources = document.getElementById('resource-audit-console');
    return topology && resources && host.contains(topology) && host.contains(resources) ? [topology, resources] : null;
  };
  const controls = roots => roots.flatMap(root => all(root, '[data-cloud-select]'));
  const links = roots => roots.flatMap(root => all(root, 'a[data-cloud-route]'));
  function validate(roots) {
    try {
      const [topology, resources] = roots, source = JSON.parse(topology.dataset.cloudSource), records = new Map(), routes = new Map(), counts = {region: 0, workload: 0, resource: 0, gate: 0};
      if (topology.dataset.cloudRoot !== 'topology' || resources.dataset.cloudRoot !== 'resources' || all(host, '[data-cloud-root]').length !== 2 || all(host, '#cockpit-panel-topology').length !== 1 || all(host, '#resource-audit-console').length !== 1
        || !Array.isArray(source) || source.length !== 5 || !['tenant-admin', 'corporate-office', 'healthcare-clinician'].includes(source[0]) || source.slice(0, 3).some(value => typeof value !== 'string' || !value)
        || source[0] !== topology.dataset.cloudContext || source[1] !== topology.dataset.cloudLabel || source[2] !== topology.dataset.cloudHubSource || !['ssr', 'island'].includes(topology.dataset.cloudMode)
        || ['cloudSource', 'cloudContext', 'cloudLabel', 'cloudHubSource', 'cloudMode'].some(name => resources.dataset[name] !== topology.dataset[name])
        || resources.dataset.aeContext !== source[0] || resources.dataset.aeHubSource !== source[2] || resources.dataset.aeMode !== topology.dataset.cloudMode
        || !['true', 'false'].includes(resources.dataset.aeSourceAvailable) || !['true', 'false'].includes(resources.dataset.aeAvailable)
        || !Array.isArray(source[3]) || source[3].length !== 17 || !Array.isArray(source[4])) return null;
      for (const record of source[3]) {
        if (!Array.isArray(record) || record.length !== 2) return null;
        const [token, pairs] = record;
        const family = token?.split(':')[0], fields = names[family];
        if (!fields || !token.split(':')[1] || records.has(token) || !Array.isArray(pairs) || pairs.length !== fields.length
          || pairs.some((pair, index) => !Array.isArray(pair) || pair.length !== 2 || pair[0] !== fields[index] || typeof pair[1] !== 'string' || !pair[1])) return null;
        const values = new Map(pairs), identity = values.get(family === 'region' ? 'Region' : family === 'gate' ? 'Label' : 'Key');
        if (token !== `${family}:${identity}`) return null;
        records.set(token, pairs); counts[family]++;
      }
      if (JSON.stringify(counts) !== JSON.stringify({region: 3, workload: 4, resource: 6, gate: 4})) return null;
      const seen = new Set();
      for (const node of controls(roots)) {
        const token = node.dataset.cloudSelect, pairs = records.get(token), family = token?.split(':')[0], values = new Map(pairs || []), fields = all(node.closest('[data-cloud-record-row]') || node, '[data-cloud-value]');
        const panel = family === 'resource' ? resources.querySelector('#resource-panel-inventory') : family === 'gate' ? resources.querySelector('#resource-panel-gates') : topology;
        if (!pairs || seen.has(token) || !panel?.contains(node) || node.dataset.cloudFocus !== token || JSON.stringify(JSON.parse(node.dataset.cloudRecord)) !== JSON.stringify(pairs)
          || fields.some(value => value.textContent !== values.get(value.dataset.cloudValue)) || new Set(fields.map(value => value.dataset.cloudValue)).size !== fields.length
          || visible[family].some(name => !fields.some(value => value.dataset.cloudValue === name))) return null;
        seen.add(token);
      }
      if (seen.size !== records.size) return null;
      const cards = all(resources, '[data-cloud-gate]'), gates = source[3].filter(record => record[0].startsWith('gate:'));
      if (cards.length !== 4) return null;
      for (const [index, [token, pairs]] of gates.entries()) {
        const values = new Map(pairs), card = cards[index], progress = card.querySelector('[role="progressbar"]');
        if (card.dataset.cloudGate !== values.get('Label') || values.get('Progress') !== ['92%', '74%', '61%', '48%'][index]
          || names.gate.some(name => card.querySelector(`[data-cloud-value="${name}"]`)?.textContent !== values.get(name))
          || progress?.getAttribute('aria-valuenow') !== values.get('Progress').slice(0, -1) || progress.getAttribute('style')?.replace(/;$/, '') !== `--bar: ${values.get('Progress')}`
          || progress.getAttribute('aria-label') !== `Sample ${values.get('Label')} gate progress: ${values.get('Progress')}; not measured readiness`) return null;
      }
      const bars = all(topology, '.ops-release-lanes [role="progressbar"]');
      if (bars.length !== 4 || bars.some((bar, index) => { const values = new Map(gates[index][1]); return bar.getAttribute('aria-valuenow') !== values.get('Progress').slice(0, -1)
        || bar.getAttribute('style')?.replace(/;$/, '') !== `--bar: ${values.get('Progress')}` || bar.getAttribute('aria-label') !== `Sample ${values.get('Label')}: ${values.get('Progress')}; not measured readiness`; })) return null;
      for (const route of source[4]) {
        if (!Array.isArray(route) || route.length !== 3 || typeof route[0] !== 'string' || !route[0] || routes.has(route[0]) || typeof route[1] !== 'string' || !route[1] || route[2] !== null && !['Mail', 'Community'].includes(route[2])) return null;
        routes.set(route[0], route);
      }
      const routed = new Set();
      for (const link of links(roots)) {
        const route = routes.get(link.dataset.cloudRoute), href = link.getAttribute('href');
        if (!route || routed.has(route[0]) || link.dataset.cloudTarget !== route[1] || (link.dataset.cloudChannel || null) !== route[2]
          || (route[2] ? href !== null && href !== `#${route[1]}` : href !== `#${route[1]}`)) return null;
        routed.add(route[0]);
      }
      if (routed.size !== routes.size || routes.size !== (source[0] === 'healthcare-clinician' ? 28 : 31)) return null;
      for (const scope of ['topology', 'resources', 'gates']) {
        const root = scope === 'topology' ? topology : resources, inspection = root.querySelector(`[data-cloud-inspection="${scope}"]`), slots = inspection && all(inspection, '[data-cloud-inspect-field]');
        if (!inspection || all(root, `[data-cloud-inspection="${scope}"]`).length !== 1 || all(inspection, '[data-cloud-inspect-title]').length !== 1 || all(inspection, `[data-cloud-close="${scope}"]`).length !== 1 || slots.length !== 9
          || slots.some((slot, index) => slot.dataset.cloudInspectField !== String(index) || all(slot, '[data-cloud-inspect-label]').length !== 1 || all(slot, '[data-cloud-inspect-value]').length !== 1)
          || all(root, `[data-cloud-status="${scope}"]`).length !== 1 || all(root, `[data-cloud-availability="${scope}"]`).length !== 1) return null;
      }
      const toolbar = all(resources, '[data-resource-search], [data-resource-filter], [data-resource-action]');
      if (toolbar.length !== 7 || all(resources, '[data-resource-search]').length !== 1 || all(resources, '[data-resource-action]').map(node => node.dataset.resourceAction).join() !== 'refresh,export'
        || all(resources, '[data-resource-filter]').map(node => node.dataset.resourceFilter).join() !== 'all,attention,review,active' || all(resources, '[data-resource-status]').length !== 1 || all(resources, '[data-cloud-empty]').length !== 1) return null;
      return {source, records, routes};
    } catch { return null; }
  }
  function workspace() {
    const root = document.getElementById('work-hub')?.querySelector('.communications-hub[data-hub-source]');
    return root && host.contains(root) && root.dataset.hubContext === current.source[0] && root.dataset.hubSource === current.source[2] && root.dataset.hubMode === 'island' && current.mode === 'island' ? root : null;
  }
  const shown = node => node && !node.disabled && !node.closest('[hidden]') && node.getClientRects().length > 0;
  function channelAvailable(channel) { const root = workspace(), tab = root?.querySelector(`#hub-tab-${channel.toLowerCase()}`), composer = root?.querySelector('.hub-composer textarea'); return shown(tab) && shown(composer) && root.querySelector('.hub-list') && root.querySelector('.hub-detail') ? root : null; }
  function channelReportCurrent(report, committed = false) {
    const roots = owners();
    return report && current?.channelReport === report && current.verified && current.available && current.key === report.key && current.mode === report.mode
      && roots?.every((root, index) => root === report.roots[index] && root === current.roots[index] && root.dataset.cloudSource === report.key && root.dataset.cloudMode === report.mode)
      && roots[1].dataset.aeSourceAvailable === 'true' && roots[1].dataset.aeAvailable === 'true' && workspace() === report.hub
      && document.getElementById(report.target) === report.destination && host.contains(report.destination) && report.link.isConnected
      && report.roots.some(root => root.contains(report.link)) && report.link.href === report.url && report.status === report.link.closest('[data-cloud-root]')?.querySelector('[data-cloud-status]')
      && (location.href === report.url || !committed && report.result === null && location.href === report.origin);
  }
  function toolbar() {
    const root = current.roots[1], selected = all(root, '.resource-tabs [role="tab"][aria-selected="true"]');
    if (selected.length !== 1 || selected[0].dataset.resourceTab === 'audit') return;
    const inventory = selected[0].dataset.resourceTab === 'inventory', enabled = inventory && current.available;
    for (const node of all(root, '[data-resource-search], [data-resource-filter], [data-resource-action]')) {
      const exportOnly = node.dataset.resourceAction === 'export', off = !enabled || exportOnly;
      const reason = !inventory ? 'Gates toolbar unavailable; inspect illustrative gates below.' : !current.available ? 'Native inventory sample unavailable; local controls disabled.' : exportOnly ? 'Sample export unavailable; no file created.' : '';
      disabled(node, off); attr(node, 'aria-disabled', off); attr(node, 'title', reason);
      if (node.matches('[data-resource-search]') && node.value !== current.query) node.value = current.query;
      if (node.matches('[data-resource-filter]')) attr(node, 'aria-pressed', node.dataset.resourceFilter === current.filter);
    }
    text(root.querySelector('[data-resource-status]'), inventory ? current.available ? `${current.count} of 6 sample resources visible · ${current.filter} filter` : 'Inventory sample unavailable · no live output created.' : 'Gates toolbar unavailable · sample inspection only; no admission or output created.');
  }
  function apply() {
    if (!channelReportCurrent(current.channelReport)) current.channelReport = null;
    let count = 0;
    for (const node of controls(current.roots)) {
      const token = node.dataset.cloudSelect, pairs = current.records.get(token) || [], values = new Map(pairs), family = token.split(':')[0], row = node.closest('[data-cloud-record-row]');
      if (family === 'resource' && row) { row.hidden = current.available && (!(current.filter === 'all' || current.filter === values.get('State')) || !normalize(pairs.map(pair => pair[1]).join(' ')).includes(normalize(current.query))); if (!row.hidden) count++; }
      const selected = current.selected[family] === token;
      disabled(node, !current.available); attr(node, 'aria-pressed', selected && current.available); attr(node, 'aria-expanded', selected && current.open === (family === 'region' || family === 'workload' ? 'topology' : family === 'resource' ? 'resources' : 'gates') && current.available);
    }
    current.count = count; const [topology, resources] = current.roots;
    const empty = resources.querySelector('[data-cloud-empty]'); if (empty) empty.hidden = !current.available || count !== 0;
    for (const scope of ['topology', 'resources', 'gates']) {
      const root = scope === 'topology' ? topology : resources, inspection = root.querySelector(`[data-cloud-inspection="${scope}"]`), token = current.inspect[scope], pairs = current.records.get(token) || [];
      if (inspection) { inspection.hidden = !current.available || current.open !== scope || !pairs.length; if (!inspection.hidden) { text(inspection.querySelector('[data-cloud-inspect-title]'), `Illustrative ${token}`); all(inspection, '[data-cloud-inspect-field]').forEach((slot, index) => { const pair = pairs[index]; slot.hidden = !pair; if (pair) { text(slot.querySelector('[data-cloud-inspect-label]'), pair[0]); text(slot.querySelector('[data-cloud-inspect-value]'), pair[1]); } }); } }
      const status = root.querySelector(`[data-cloud-status="${scope}"]`), report = current.channelReport;
      text(status, report?.status === status && report.result ? report.result : !current.available ? 'Native sample unavailable; local inspection and routes disabled.' : token ? `Illustrative ${token}; no live verification or operation.` : 'Select an illustrative sample; live operations unavailable.');
      all(root, `[data-cloud-close="${scope}"]`).forEach(node => disabled(node, !current.available));
    }
    all(topology, '[data-cloud-workload-detail]').forEach(node => text(node, new Map(current.records.get(current.selected.workload) || []).get(node.dataset.cloudWorkloadDetail) || 'Unavailable'));
    all(resources, '[data-cloud-gate]').forEach(card => card.classList.toggle('cloud-selected', current.available && current.selected.gate === `gate:${card.dataset.cloudGate}`));
    current.roots.flatMap(root => all(root, '[data-cloud-effect]')).forEach(node => disabled(node, true));
    const inspectRegion = topology.querySelector('[data-cloud-inspect-region]'); if (inspectRegion) { disabled(inspectRegion, !current.available); attr(inspectRegion, 'aria-expanded', current.available && current.open === 'topology' && current.inspect.topology === current.selected.region); }
    for (const link of links(current.roots)) {
      const channel = link.dataset.cloudChannel, ready = current.available && (!channel || channelAvailable(channel)); attr(link, 'aria-disabled', !ready);
      if (channel) { if (ready) attr(link, 'href', `#${link.dataset.cloudTarget}`); else if (link.hasAttribute('href')) link.removeAttribute('href'); }
    }
    toolbar();
  }
  function synchronize() {
    const roots = owners();
    if (!roots) { if (current) { current.available = false; current.channelReport = null; } return; }
    const verified = validate(roots), key = roots[0].dataset.cloudSource, available = !!verified && roots[1].dataset.aeSourceAvailable === 'true' && roots[1].dataset.aeAvailable === 'true';
    const same = current?.key === key && current.verified && verified, previous = current;
    const defaults = verified ? verified.source[3] : [];
    current = {roots, key, verified: !!verified, available, source: verified?.source || [], records: verified?.records || new Map(), mode: roots[0].dataset.cloudMode,
      query: same ? previous.query : '', filter: same ? previous.filter : 'all', selected: same ? previous.selected : Object.fromEntries(['region', 'workload'].map(family => [family, defaults.find(record => record[0].startsWith(family + ':'))?.[0]])),
      inspect: same ? previous.inspect : {}, open: same ? previous.open : null, focus: same && available ? previous.focus : null,
      tab: same ? previous.tab : null, restoration: same ? previous.restoration : true, restoreTab: same && previous.restoration && previous.roots[1] !== roots[1] ? previous.tab : same ? previous.restoreTab : null,
      channelReport: same && available && previous.available && previous.roots.every((root, index) => root === roots[index]) && previous.mode === roots[0].dataset.cloudMode ? previous.channelReport : null};
    if (current.restoreTab && current.available && document.hasFocus()) {
      const tab = roots[1].querySelector(`#resource-tab-${current.restoreTab}`), active = document.activeElement;
      if (active?.isConnected && active !== document.body && !roots.some(root => root.contains(active))) current.restoreTab = null;
      else if (shown(tab)) { restoring = true; if (tab.getAttribute('aria-selected') !== 'true') tab.click(); restoring = false;
        if (tab.getAttribute('aria-selected') === 'true') current.restoreTab = null;
        if (active?.isConnected && active !== document.body && document.activeElement === tab) active.focus({preventScroll: true}); }
    }
    if (!current.restoreTab) current.tab = roots[1].querySelector('.resource-tabs [role="tab"][aria-selected="true"]')?.dataset.resourceTab;
    apply();
    const saved = current.focus;
    if (saved && !saved.node.isConnected) {
      const active = document.activeElement;
      current.focus = null;
      if (document.hasFocus() && location.href === saved.url && (!active || active === document.body || active.id === `resource-tab-${current.tab}`)) {
        const node = controls(roots).find(node => node.dataset.cloudFocus === saved.token); if (shown(node)) { restoring = true; node.focus({preventScroll: true}); restoring = false; current.focus = {...saved, node}; }
      }
    }
  }
  function schedule() { if (queued) return; queued = true; queueMicrotask(() => { queued = false; synchronize(); }); }
  function clearChannelReport() { if (current?.channelReport) { current.channelReport = null; schedule(); } }
  function target(event) {
    if (!(event.target instanceof Element) || !event.target.isConnected) return null;
    const roots = owners(), root = event.target.closest('[data-cloud-root]'); if (!roots?.includes(root)) return null;
    synchronize(); return event.target;
  }
  function route(link) {
    const destination = document.getElementById(link.dataset.cloudTarget); if (!destination || !host.contains(destination)) return false;
    if (link.dataset.cloudChannel) {
      const root = channelAvailable(link.dataset.cloudChannel), tab = root?.querySelector(`#hub-tab-${link.dataset.cloudChannel.toLowerCase()}`); if (!root || !tab) return false;
      const channel = link.dataset.cloudChannel, report = {key: current.key, roots: current.roots, mode: current.mode, hub: root, destination, target: link.dataset.cloudTarget, link,
        status: link.closest('[data-cloud-root]').querySelector('[data-cloud-status]'), url: link.href, origin: location.href, result: null};
      current.channelReport = report; tab.click();
      requestAnimationFrame(() => {
        if (!channelReportCurrent(report, true) || !validate(report.roots)) { if (current?.channelReport === report) { current.channelReport = null; apply(); } return; }
        const fresh = workspace(), selected = fresh.querySelector(`#hub-tab-${channel.toLowerCase()}`);
        const ready = shown(selected) && selected.getAttribute('aria-selected') === 'true' && fresh.querySelector('[data-comms-product-board]')?.dataset.commsBoardSurface === channel
          && fresh.querySelector('.hub-list') && fresh.querySelector('.hub-detail') && shown(fresh.querySelector('.hub-composer textarea'));
        report.result = ready ? `${channel} existing sample opened; no live operation or output created.` : `${channel} sample unavailable; no live operation or output created.`; apply();
      });
    } else if (current.mode === 'ssr') {
      const panel = destination.closest('[role="tabpanel"]');
      if (panel) { const tab = document.getElementById(panel.getAttribute('aria-labelledby')), group = tab?.closest('[role="tablist"]');
        if (!shown(tab) || tab.parentElement !== group || tab.getAttribute('aria-controls') !== panel.id || group.classList.contains('hub-tabs')) return false;
        tab.click(); if (tab.getAttribute('aria-selected') !== 'true' || !panel.classList.contains('active')) return false; }
      if (document.hasFocus()) { if (!destination.hasAttribute('tabindex')) destination.setAttribute('tabindex', '-1'); destination.focus({preventScroll: true}); destination.scrollIntoView({block: 'start'}); }
    }
    current.focus = null; return true;
  }
  document.addEventListener('input', event => { if (event.isTrusted) clearChannelReport(); const value = event.target?.value, node = target(event); if (node) clearChannelReport(); if (node?.matches('[data-resource-search]') && current.available && current.tab === 'inventory') { current.query = value; current.restoration = true; apply(); schedule(); } });
  document.addEventListener('click', event => {
    if (event.isTrusted) clearChannelReport();
    const node = target(event); if (!node || event.defaultPrevented || event.button !== 0 || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    clearChannelReport();
    const action = node.closest('[data-cloud-select], [data-cloud-close], [data-cloud-inspect-region], [data-cloud-route], [data-cloud-effect], [data-resource-filter], [data-resource-action]');
    if (!action) return;
    if (action.matches('[data-resource-filter], [data-resource-action]') && current.tab === 'audit') return;
    if (!current.available || action.matches('[data-cloud-effect]') || action.dataset.resourceAction === 'export') { event.preventDefault(); return; }
    if (action.matches('[data-resource-filter], [data-resource-action]')) { if (current.tab !== 'inventory') { event.preventDefault(); return; } current.query = action.dataset.resourceAction ? '' : current.query; current.filter = action.dataset.resourceFilter || 'all'; }
    else if (action.matches('[data-cloud-select]')) { const token = action.dataset.cloudSelect, family = token.split(':')[0], scope = ['region', 'workload'].includes(family) ? 'topology' : family === 'resource' ? 'resources' : 'gates'; current.selected[family] = token; current.inspect[scope] = token; current.open = scope; }
    else if (action.matches('[data-cloud-inspect-region]')) { current.inspect.topology = current.selected.region; current.open = 'topology'; }
    else if (action.matches('[data-cloud-close]')) { const token = current.inspect[action.dataset.cloudClose]; current.open = null; const control = controls(current.roots).find(node => node.dataset.cloudSelect === token); if (shown(control) && document.hasFocus()) control.focus({preventScroll: true}); }
    else if (!route(action)) { event.preventDefault(); text(action.closest('[data-cloud-root]').querySelector('[data-cloud-status]'), 'Existing sample destination unavailable; no output created.'); return; }
    current.restoration = true; apply(); schedule();
  }, true);
  document.addEventListener('focusin', event => { if (restoring) return; if (current && event.target !== document.body && event.target !== current.focus?.node) { current.focus = null; current.restoreTab = null; current.restoration = false; } const node = target(event)?.closest('[data-cloud-focus]'); if (node && current?.available) { current.focus = {node, token: node.dataset.cloudFocus, url: location.href}; current.restoration = true; } else if (event.target.closest?.('.resource-tabs') && current) current.restoration = true; });
  for (const name of ['pointerdown', 'keydown']) document.addEventListener(name, event => { if (event.isTrusted && current) { clearChannelReport(); current.restoreTab = null; current.restoration = false; if (event.target !== current.focus?.node) current.focus = null; } }, true);
  document.addEventListener('keydown', event => { const node = target(event); if (node && event.key === 'Escape' && current.available && current.open) { event.preventDefault(); current.open = null; apply(); } });
  for (const name of ['blur', 'hashchange', 'popstate']) window.addEventListener(name, () => { if (current) { if (name === 'blur' || current.channelReport && location.href !== current.channelReport.url) clearChannelReport(); current.focus = null; current.restoreTab = null; current.restoration = false; } });
  new MutationObserver(records => {
    if (!records.some(record => record.type === 'childList' || record.type === 'attributes' || record.target.parentElement?.closest('[data-cloud-value]'))) return;
    schedule();
  }).observe(host, {childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ['data-cloud-root', 'data-cloud-source', 'data-cloud-context', 'data-cloud-label', 'data-cloud-hub-source', 'data-cloud-mode', 'data-cloud-select', 'data-cloud-record', 'data-cloud-focus', 'data-cloud-value', 'data-cloud-route', 'data-cloud-target', 'data-cloud-channel', 'data-cloud-gate', 'data-cloud-inspection', 'data-cloud-inspect-field', 'data-cloud-inspect-title', 'data-cloud-inspect-label', 'data-cloud-inspect-value', 'data-cloud-close', 'data-cloud-availability', 'data-cloud-status', 'data-cloud-empty', 'data-resource-search', 'data-resource-filter', 'data-resource-action', 'data-resource-status', 'data-ae-source-available', 'data-ae-available', 'data-ae-context', 'data-ae-hub-source', 'data-ae-mode', 'aria-selected', 'aria-controls', 'aria-labelledby', 'aria-valuenow', 'aria-label', 'style', 'href', 'data-hub-context', 'data-hub-source', 'data-hub-mode', 'data-comms-board-surface', 'disabled']});
  synchronize();
})();
"##;
