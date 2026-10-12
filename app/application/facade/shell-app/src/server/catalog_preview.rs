#[path = "catalog_preview_routes.rs"]
mod routes;

pub(super) fn script() -> String {
    format!("{STATE}{}{EVENTS}", routes::script())
}

const STATE: &str = r##"
(() => {
  const host = document.getElementById('dashboard-island-root');
  if (!host) return;
  let current = null, handoff = null, queued = false, dispatching = false, report = null;
  const boundSources = new WeakMap();
  const fields = ['Capability', 'Module', 'Category', 'Description', 'Action', 'Sample health',
    'Sample owner', 'Sample dependency', 'Sample priority', 'Destination', 'Channel', 'Availability'];
  const groups = {Control:'control', Cloud:'cloud', Operations:'operations', Trust:'trust',
    Corporate:'corporate', Daily:'daily', Workflow:'workflow', 'No-code':'no-code', Healthcare:'healthcare', Data:'data'};
  const health = {sample:'Sample only', review:'Review example', attention:'Attention example', receipt:'Receipt example'};
  const all = (root, selector) => [...root.querySelectorAll(selector)];
  const normalize = value => String(value).normalize('NFC').trim().toLocaleLowerCase();
  const visible = node => node?.isConnected && node.getClientRects().length > 0;
  const text = (node, value) => { if (node && node.textContent !== String(value)) node.textContent = String(value); };
  const attr = (node, name, value) => { if (node.getAttribute(name) !== String(value)) node.setAttribute(name, String(value)); };
  const disable = (node, value) => { if (node.disabled !== value) node.disabled = value; };
  const owner = () => {
    const roots = all(host, '#service-catalog');
    return roots.length === 1 && all(host, '[data-catalog-root]').length === 1 ? roots[0] : null;
  };
  function validate(root) {
    try {
      if (!root || root.dataset.catalogRoot !== 'true' || root.dataset.catalogAdmitted !== 'true') return null;
      const key = root.dataset.catalogSource, source = JSON.parse(key), records = new Map();
      const context = source[0], mode = root.dataset.catalogMode, permitted = admittedIds[context];
      if (boundSources.has(root) && boundSources.get(root) !== key) return null;
      if (!Array.isArray(source) || source.length !== 4 || !Object.hasOwn(admittedIds, context)
        || source.slice(0,3).some(value => typeof value !== 'string' || !value)
        || source[1] !== root.dataset.catalogLabel || source[2] !== root.dataset.catalogHubSource
        || context !== root.dataset.catalogContext || !['ssr','island'].includes(mode)
        || !['true','false'].includes(root.dataset.catalogAvailable) || !Array.isArray(source[3])
        || source[3].some(record => !Array.isArray(record) || record.length !== 2)) return null;
      const rows = all(root, '[data-catalog-module]');
      if (rows.length !== source[3].length || rows.length !== permitted.length) return null;
      for (const [id, pairs] of source[3]) {
        if (records.has(id) || !permitted.includes(id) || !knownIdentity(id) || !Array.isArray(pairs) || pairs.length !== fields.length
          || pairs.some((pair,index) => !Array.isArray(pair) || pair.length !== 2 || pair[0] !== fields[index] || typeof pair[1] !== 'string')
          || pairs[0][1] !== id) return null;
        const matches = rows.filter(row => row.dataset.catalogModule === id), values = new Map(pairs);
        if (matches.length !== 1) return null;
        const row = matches[0], route = cardRoute(id);
        if (JSON.stringify(JSON.parse(row.dataset.catalogRecord)) !== JSON.stringify(pairs)
          || row.dataset.catalogGroup !== groups[values.get('Category')]
          || !['sample','review','attention','receipt'].includes(row.dataset.catalogState)
          || values.get('Sample health') !== health[row.dataset.catalogState]
          || values.get('Destination') !== route[0] || values.get('Channel') !== route[1]) return null;
        for (const name of ['Module','Category','Description','Sample health','Sample owner','Sample dependency','Sample priority']) {
          const nodes = all(row, `[data-catalog-field="${name}"]`);
          if (nodes.length !== 1 || nodes[0].textContent !== values.get(name)) return null;
        }
        const details = all(row, '[data-catalog-detail]'), pins = all(row, '[data-catalog-pin]');
        if (details.length !== 1 || details[0].dataset.catalogDetail !== id || pins.length !== 1 || pins[0].dataset.catalogPin !== id
          || details[0].dataset.sidepeekId !== 'CAT-OYATIE-' + values.get('Module').toUpperCase().replaceAll(' ', '-')) return null;
        if (!validCardRoute(row, id, values)) return null;
        records.set(id, {id, pairs, values, row});
      }
      const categories = new Set([...records.values()].map(record => record.row.dataset.catalogGroup));
      const filters = all(root, '[data-catalog-filter]'), filterIds = filters.map(node => node.dataset.catalogFilter);
      if (filters.length !== categories.size + 1 || new Set(filterIds).size !== filters.length
        || !filterIds.includes('all') || [...categories].some(group => !filterIds.includes(group)) || !validConnectedLinks(root)) return null;
      const tokens = all(root, '[data-catalog-focus]').map(node => node.dataset.catalogFocus);
      if (new Set(tokens).size !== tokens.length || all(root, '[data-catalog-search]').length !== 1
        || all(root, '[data-catalog-inspection]').length !== 1 || all(root, '[data-catalog-inspect-field]').length !== fields.length
        || all(root, '[data-catalog-close]').length !== 1 || all(root, '[data-catalog-status]').length !== 1) return null;
      const dashboard = root.closest('[data-island="render-envelope-dashboard"]');
      const picker = dashboard?.querySelector('[data-sample-context-requested]');
      const available = root.dataset.catalogAvailable === 'true' && dashboard?.getAttribute('aria-busy') === 'false'
        && dashboard.dataset.fragmentNavigation === mode
        && picker?.dataset.sampleContextRequested === context && picker.dataset.sampleContextDisplayed === context;
      boundSources.set(root, key);
      return {root, key, source, records, mode, available};
    } catch { return null; }
  }
  function controls(root, available) {
    all(root, 'button, input').forEach(node => disable(node, !available || node.hasAttribute('data-catalog-unavailable')
      || !!node.closest('.catalog-row-actions') && !node.hasAttribute('data-catalog-pin') && !node.hasAttribute('data-catalog-action')));
    all(root, 'a[data-catalog-route], a[data-catalog-link]').forEach(node => {
      const enabled = available && destinationFor(node) !== null;
      attr(node, 'aria-disabled', !enabled); attr(node, 'tabindex', enabled ? '0' : '-1');
    });
    all(root, '[data-catalog-module]').forEach(row => {
      const anchor = row.querySelector('[data-catalog-route]'), reason = row.querySelector('.catalog-open-reason');
      if (anchor && reason) {
        reason.hidden = !available || destinationFor(anchor) !== null;
        text(reason, 'Existing shell sample destination unavailable in this render.');
      }
    });
    all(root, '[data-catalog-link-reason]').forEach(reason => {
      const links = all(root, '[data-catalog-link]').filter(anchor => anchor.dataset.catalogLink === reason.dataset.catalogLinkReason);
      const anchor = links.length === 1 ? links[0] : null;
      const bound = anchor && connectedReason(root, anchor) === reason;
      const missing = available && bound && destinationFor(anchor) === null;
      reason.hidden = !missing;
      text(reason, missing ? `${anchor.textContent.trim()} destination unavailable in this render.` : '');
      if (bound) attr(anchor, 'aria-describedby', missing ? 'catalog-availability ' + reason.id : 'catalog-availability');
    });
  }
  function apply() {
    if (!current) return;
    const {root, records, available} = current;
    controls(root, available);
    const search = root.querySelector('[data-catalog-search]');
    if (search.value !== current.query) search.value = current.query;
    let count = 0;
    for (const record of records.values()) {
      const {id, row, pairs} = record;
      row.hidden = available && (!(current.category === 'all' || current.category === row.dataset.catalogGroup)
        || current.attention && row.dataset.catalogState !== 'attention'
        || !normalize(pairs.map(pair => pair[1]).join(' ')).includes(normalize(current.query)));
      if (!row.hidden) count++;
      row.classList.toggle('pinned', current.pins.has(id));
      const pin = row.querySelector('[data-catalog-pin]');
      attr(pin, 'aria-pressed', available && current.pins.has(id)); text(pin, current.pins.has(id) ? 'Unpin locally' : 'Pin locally');
      attr(row.querySelector('[data-catalog-detail]'), 'aria-expanded', available && current.open && current.selected === id && !row.hidden);
    }
    all(root, '[data-catalog-filter]').forEach(node => {
      const pressed = current.category === node.dataset.catalogFilter;
      attr(node, 'aria-pressed', pressed); node.classList.toggle('active', pressed);
    });
    attr(root.querySelector('[data-catalog-attention]'), 'aria-pressed', current.attention);
    const inspection = root.querySelector('[data-catalog-inspection]'), selected = records.get(current.selected);
    inspection.hidden = !available || !current.open || !selected || selected.row.hidden;
    if (!inspection.hidden) {
      text(root.querySelector('[data-catalog-detail-title]'), `Illustrative ${selected.values.get('Module')}`);
      all(root, '[data-catalog-inspect-field]').forEach((slot,index) => {
        slot.hidden = false; text(slot.querySelector('dt'), selected.pairs[index][0]); text(slot.querySelector('dd'), selected.pairs[index][1]);
      });
    }
    root.querySelector('[data-catalog-empty]').hidden = !available || count !== 0;
    text(root.querySelector('[data-catalog-status]'), available
      ? `${count} of ${records.size} sample cards · ${current.category} category${current.attention ? ' · attention examples' : ''}`
      : 'Sample source unavailable; local controls disabled.');
  }
  function nativeGap(previous) {
    if (!previous?.available || previous.root.isConnected || !host.isConnected
      || all(host, '#service-catalog, [data-catalog-root], .fetch-error').length) return false;
    const frames = all(host, '[data-island="render-envelope-dashboard"]');
    const pickers = all(host, '[data-sample-context-requested]');
    return frames.length === 0 && host.childElementCount === 0 || frames.length === 1 && pickers.length === 1
      && frames[0].getAttribute('aria-busy') === 'true' && frames[0].dataset.fragmentNavigation === 'island'
      && pickers[0].dataset.sampleContextRequested === previous.source[0]
      && pickers[0].dataset.sampleContextDisplayed === 'none';
  }
  function cancelFocus() {
    const state = current || handoff;
    if (state) state.focusIntent = null;
  }
  function synchronize() {
    const root = owner(), next = validate(root), previous = current || handoff;
    if (!next) {
      all(host, '#service-catalog, [data-catalog-root]').forEach(node => controls(node, false));
      handoff = nativeGap(previous) ? previous : null;
      current = null; report = null; return;
    }
    handoff = null;
    const same = previous?.key === next.key && previous.available && next.available;
    const replaced = previous && previous.root !== root;
    const state = same ? previous : {query:'', category:'all', attention:false, pins:new Set(), selected:null, open:false, focusIntent:null, scroll:0};
    current = {...state, ...next};
    if (report && (!same || replaced && previous.mode !== next.mode)) report = null;
    apply();
    settleReport();
    if (same && replaced) {
      const table = root.querySelector('.catalog-table-shell');
      table.scrollLeft = previous.scroll;
      const intent = previous.focusIntent ? {...previous.focusIntent} : null;
      if (intent?.token === 'search') {
        const input = previous.root.querySelector('[data-catalog-search]');
        intent.start = input.selectionStart ?? 0; intent.end = input.selectionEnd ?? 0;
      }
      current.focusIntent = null;
      const active = document.activeElement;
      if (intent?.key === next.key && intent.url === location.href && document.hasFocus()
        && (!active || active === document.body || !active.isConnected)) {
        const target = all(root, '[data-catalog-focus]').find(node => node.dataset.catalogFocus === intent.token);
        if (visible(target) && !target.disabled && target.getAttribute('aria-disabled') !== 'true') {
          target.focus({preventScroll:true});
          if (target === root.querySelector('[data-catalog-search]')) target.setSelectionRange(intent.start, intent.end);
        }
      }
    }
  }
  function owned(event) {
    if (!(event.target instanceof Element) || !event.target.isConnected || !owner()?.contains(event.target)) return null;
    synchronize();
    return current?.root.contains(event.target) ? event.target : null;
  }
  function close() {
    current.open = false; apply();
    const selected = current.records.get(current.selected)?.row.querySelector('[data-catalog-detail]');
    const target = visible(selected) ? selected : current.root.querySelector('[data-catalog-search]');
    if (document.hasFocus() && visible(target) && !target.disabled) target.focus({preventScroll:true});
  }
"##;

const EVENTS: &str = r##"
  document.addEventListener('input', event => {
    const candidate = event.target instanceof Element ? event.target.closest('[data-catalog-search]') : null;
    if (!candidate) return;
    const query = candidate.value, start = candidate.selectionStart ?? 0, end = candidate.selectionEnd ?? 0;
    const input = owned(event)?.closest('[data-catalog-search]');
    if (!input || !current.available || input.disabled) return;
    current.query = query; apply(); input.setSelectionRange(start, end);
    if (current.focusIntent?.token === 'search') Object.assign(current.focusIntent, {start, end});
  });
  document.addEventListener('click', event => {
    if (dispatching) return;
    if (!(event.target instanceof Element) || !event.target.closest('#service-catalog')) {
      if (event.isTrusted) cancelFocus();
      report = null; return;
    }
    const target = owned(event);
    if (!target || !current?.available) { event.preventDefault(); event.stopImmediatePropagation(); return; }
    const action = target?.closest('button, a[data-catalog-route], a[data-catalog-link]');
    if (!action) return;
    if (!current?.available || action.disabled || action.hasAttribute('data-catalog-unavailable') || action.getAttribute('aria-disabled') === 'true') {
      event.preventDefault(); event.stopImmediatePropagation(); return;
    }
    if (event.button !== 0 || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    if (action.hasAttribute('data-catalog-detail')) {
      current.selected = action.dataset.catalogDetail; current.open = true; apply();
    } else if (action.hasAttribute('data-catalog-pin')) {
      const id = action.dataset.catalogPin;
      if (current.pins.has(id)) current.pins.delete(id); else current.pins.add(id);
      apply();
    } else if (action.hasAttribute('data-catalog-filter')) {
      current.category = action.dataset.catalogFilter; apply();
    } else if (action.hasAttribute('data-catalog-attention')) {
      current.attention = !current.attention; apply();
    } else if (action.hasAttribute('data-catalog-reset')) {
      current.query = ''; current.category = 'all'; current.attention = false;
      current.pins.clear(); current.selected = null; current.open = false; apply();
    } else if (action.hasAttribute('data-catalog-close')) close();
    else if (action.hasAttribute('data-catalog-route') || action.hasAttribute('data-catalog-link')) route(action, event);
    else if (!(action.dataset.catalogAction === 'open' && action.closest('[data-catalog-module]')?.dataset.catalogModule === 'foundry-ontology')) {
      event.preventDefault(); event.stopImmediatePropagation();
    }
  }, true);
  document.addEventListener('keydown', event => {
    if (event.key === 'Escape' && owned(event) && current.available && current.open) { event.preventDefault(); close(); }
  });
  document.addEventListener('focusin', event => {
    if (dispatching) return;
    if (current?.root.contains(event.target)) {
      const target = event.target.closest('[data-catalog-focus]');
      if (target) current.focusIntent = {key:current.key, token:target.dataset.catalogFocus, url:location.href,
        start:target.selectionStart ?? 0, end:target.selectionEnd ?? 0};
    } else if (report && reportOwned(report) && event.target === document.getElementById(report.destination)
      && event.target.isConnected && host.contains(event.target) && all(host, '#' + report.destination).length === 1) {
      cancelFocus();
    } else if (event.target !== document.body) {
      cancelFocus();
      report = null;
    }
  });
  document.addEventListener('scroll', event => {
    if (current?.root.contains(event.target) && event.target.matches?.('.catalog-table-shell')) current.scroll = event.target.scrollLeft;
  }, true);
  window.addEventListener('hashchange', () => {
    const state = current || handoff;
    if (state?.focusIntent?.url !== location.href) cancelFocus();
    if (report && report.hash !== location.hash) report = null;
    settleReport();
  });
  new MutationObserver(() => {
    if (queued) return;
    queued = true; queueMicrotask(() => { queued = false; synchronize(); });
  }).observe(host, {childList:true, subtree:true, characterData:true, attributes:true,
    attributeFilter:['data-catalog-root','data-catalog-source','data-catalog-record','data-catalog-module','data-catalog-context',
      'data-catalog-available','data-catalog-admitted','data-catalog-mode','data-catalog-field','data-catalog-route','data-catalog-channel',
      'data-catalog-detail','data-catalog-pin','data-catalog-link','data-catalog-link-reason','data-catalog-filter','data-sidepeek-id','href','id','aria-describedby','aria-busy',
      'data-sample-context-requested','data-sample-context-displayed','disabled','aria-selected','role','aria-controls','aria-labelledby','data-comms-board-surface',
      'data-hub-source','data-hub-context','data-hub-mode']});
  synchronize();
})();
"##;

#[cfg(test)]
#[path = "catalog_preview_test.rs"]
mod tests;
