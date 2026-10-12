pub(super) fn script() -> String {
    SCRIPT.replace(
        "__SAMPLE_RECORDS__",
        &crate::app::notification_source().replace('<', "\\u003c"),
    )
}

const SCRIPT: &str = r##"
(() => {
  const source = __SAMPLE_RECORDS__, sourceKey = JSON.stringify(source);
  const prior = window.oyatieSampleNotifications;
  const inherited = prior?.snapshot();
  prior?.dispose();
  const contexts = {'tenant-admin':'Tenant admin', 'corporate-office':'Corporate office', 'healthcare-clinician':'Healthcare clinician'};
  const routes = {'work-hub':'work-hub', evidence:'evidence-spine', cloud:'cockpit-panel-topology', audit:'resource-panel-audit'};
  const all = (root, selector) => root ? [...root.querySelectorAll(selector)] : [];
  const unique = (root, selector) => { const nodes = all(root, selector); return nodes.length === 1 ? nodes[0] : null; };
  const text = (node, value) => { if (node && node.textContent !== String(value)) node.textContent = String(value); };
  const attr = (node, name, value) => { if (node && node.getAttribute(name) !== String(value)) node.setAttribute(name, String(value)); };
  const disable = (node, value) => { if (node.disabled !== value) node.disabled = value; };
  let state = inherited, bound = null, queued = false, pending = null, disposed = false;
  function context() {
    const host = unique(document, '#dashboard-island-root'), dashboard = unique(host, '[data-island="render-envelope-dashboard"]');
    const picker = unique(dashboard, '[data-sample-context-requested]'), id = picker?.dataset.sampleContextDisplayed;
    if (!host?.isConnected || !dashboard || !Object.hasOwn(contexts, id) || picker.dataset.sampleContextRequested !== id
      || dashboard.getAttribute('aria-busy') !== 'false' || !['ssr','island'].includes(dashboard.dataset.fragmentNavigation)
      || dashboard.querySelector('.fetch-error')) return null;
    return { dashboard, id, mode:dashboard.dataset.fragmentNavigation, key:JSON.stringify([id, sourceKey]) };
  }
  function owner() { return unique(document, '#notification-center'); }
  function valid(root) {
    try {
      if (!root?.isConnected || root !== owner() || root.tagName !== 'DIALOG' || root.dataset.utilityPanel !== 'notifications'
        || JSON.stringify(JSON.parse(root.dataset.notificationSource)) !== sourceKey) return false;
      const rows = all(root, '[data-activity-item]');
      if (rows.length !== source.length || rows.some((row,index) => {
        const [id,,severity,time,title,detail] = source[index];
        return row.dataset.activityItem !== id || row.dataset.activitySeverity !== severity
          || ['time','title','detail'].some((name,index) => unique(row, `[data-notification-value="${name}"]`)?.textContent !== [time,title,detail][index])
          || all(row, '[data-activity-action="mark-read"]').length !== 1;
      })) return false;
      return ['all','unread','blocking'].every(name => all(root, `[data-activity-filter="${name}"]`).length === 1)
        && Object.keys(routes).every(name => all(root, `[data-notification-route="${name}"]`).length === 1)
        && all(root, '[data-activity-filter]').length === 3 && all(root, '[data-notification-route]').length === 4
        && ['unread','total','blocking'].every(name => all(root, `[data-activity-count="${name}"]`).length === 1)
        && ['[data-activity-status]','[data-activity-empty]','[data-notification-context]','[data-activity-action="clear-read"]'].every(selector => all(root, selector).length === 1);
    } catch { return false; }
  }
  const controls = root => all(root, '[data-activity-action], [data-activity-filter], [data-notification-route]');
  function apply(root, view) {
    const available = !!view, rows = all(root, '[data-activity-item]');
    controls(root).forEach(node => disable(node, !available));
    text(root.querySelector('[data-notification-context]'), view ? `Displayed sample: ${contexts[view.id]}` : 'Displayed sample context unavailable');
    if (!view) {
      rows.forEach(row => { if (!row.hidden) row.hidden = true; });
      all(root, '[data-activity-count]').forEach(node => text(node, '—'));
      text(root.querySelector('[data-activity-status]'), 'Sample source loading, failed, missing or mismatched; local actions unavailable.');
      const empty = root.querySelector('[data-activity-empty]'); if (empty && !empty.hidden) empty.hidden = true;
    } else {
      const remaining = source.filter(record => !state.cleared.includes(record[0]));
      const counts = {total:remaining.length, unread:remaining.filter(record => !state.read.includes(record[0])).length, blocking:remaining.filter(record => record[2] === 'blocking').length};
      let shown = 0;
      rows.forEach(row => {
        const id = row.dataset.activityItem, read = state.read.includes(id);
        const hidden = state.cleared.includes(id) || state.filter === 'unread' && read || state.filter === 'blocking' && row.dataset.activitySeverity !== 'blocking';
        if (row.hidden !== hidden) row.hidden = hidden;
        if (!hidden) shown++;
        attr(row, 'data-activity-state', read ? 'read' : 'unread'); row.classList.toggle('read', read);
        const button = row.querySelector('[data-activity-action="mark-read"]'); disable(button, read || hidden);
        text(button, read ? 'Read locally' : 'Mark read locally');
        all(row, '[data-notification-route]').forEach(node => disable(node, hidden));
      });
      all(root, '[data-activity-filter]').forEach(node => { const active = node.dataset.activityFilter === state.filter;
        attr(node, 'aria-pressed', active); if (node.classList.contains('active') !== active) node.classList.toggle('active', active); });
      Object.entries(counts).forEach(([name,count]) => text(root.querySelector(`[data-activity-count="${name}"]`), count));
      disable(root.querySelector('[data-activity-action="clear-read"]'), !remaining.some(record => state.read.includes(record[0])));
      const empty = root.querySelector('[data-activity-empty]');
      text(empty, remaining.length ? 'No sample notifications match this filter.' : 'No sample notifications remain. Read samples were cleared locally.');
      if (empty.hidden !== !!shown) empty.hidden = !!shown;
      text(root.querySelector('[data-activity-status]'), `${state.message} ${shown} of ${counts.total} sample notifications shown; ${counts.unread} unread.`);
    }
    const count = view ? source.filter(record => !state.cleared.includes(record[0]) && !state.read.includes(record[0])).length : '—';
    all(document, '[data-activity-badge]').forEach(node => text(node, count));
    const invoker = unique(document, '[data-header-action="notifications"]');
    attr(invoker, 'aria-label', view ? `Open sample notifications, ${count} unread` : 'Open sample notifications, context unavailable');
  }
  function synchronize() {
    if (disposed) return null;
    const root = owner(), view = valid(root) ? context() : null;
    if (bound !== root) {
      if (bound) { bound.removeEventListener('click', click); controls(bound).forEach(node => disable(node, true)); }
      bound = root;
      if (bound) bound.addEventListener('click', click);
      pending = null;
    }
    if (view && state?.key !== view.key) {
      state = {key:view.key, read:source.filter(record => record[1]).map(record => record[0]), cleared:[], filter:'all',
        message:state ? 'Displayed sample changed; local notification choices reset.' : 'Sample examples only. Read and clear choices remain local to this displayed sample.'};
      pending = null;
    }
    if (!view) pending = null;
    if (root) apply(root, view);
    else all(document, '#notification-center').forEach(node => apply(node, null));
    return view;
  }
  function tabFor(target, view) {
    const panel = target.closest('[role="tabpanel"]'); if (!panel) return null;
    const tab = unique(view.dashboard, '#' + panel.getAttribute('aria-labelledby')), list = tab?.closest('[role="tablist"]');
    const tabs = all(list, '[role="tab"]');
    if (!tab || tab.disabled || tab.parentElement !== list || !tab.getClientRects().length || tabs.filter(node => node.getAttribute('aria-selected') === 'true').length !== 1
      || tabs.some(node => { const controlled = unique(view.dashboard, '#' + node.getAttribute('aria-controls'));
        return node.parentElement !== list || unique(view.dashboard, '#' + node.id) !== node || !controlled
          || controlled.getAttribute('role') !== 'tabpanel' || controlled.getAttribute('aria-labelledby') !== node.id; })
      || tab.getAttribute('aria-controls') !== panel.id) return false;
    return tab;
  }
  function destination(route, view) {
    try {
      const target = unique(view.dashboard, '#' + routes[route]);
      if (!target || target.closest('[hidden], [inert], [aria-hidden="true"]')) return null;
      const hub = unique(view.dashboard, '.communications-hub[data-hub-source]'), hubSource = JSON.parse(hub?.dataset.hubSource);
      if (!Array.isArray(hubSource) || hubSource.length !== 8 || hubSource[0] !== view.id || hub.dataset.hubContext !== view.id || hub.dataset.hubMode !== view.mode) return null;
      if (route === 'work-hub' && !target.contains(hub)) return null;
      const name = route === 'cloud' ? 'cloud' : ['evidence','audit'].includes(route) ? 'ae' : null;
      if (name) {
        const roots = name === 'ae' ? ['#evidence-spine','#resource-audit-console'].map(selector => unique(view.dashboard, selector))
          : ['#cockpit-panel-topology','#resource-audit-console'].map(selector => unique(view.dashboard, selector));
        const first = roots[0], sample = JSON.parse(first?.dataset[name + 'Source']);
        if (!Array.isArray(sample) || sample.length !== (name === 'ae' ? 7 : 5) || sample[0] !== view.id
          || sample[1] !== first.dataset[name + 'Label'] || sample[2] !== hub.dataset.hubSource
          || roots.some(root => !root || ['Source','Context','Label','HubSource','Mode'].some(field => root.dataset[name + field] !== first.dataset[name + field])
            || root.dataset[name + 'Context'] !== view.id || root.dataset[name + 'Mode'] !== view.mode
            || name === 'ae' && (root.dataset.aeSourceAvailable !== 'true' || root.dataset.aeAvailable !== 'true'))) return null;
      }
      const tab = tabFor(target, view);
      if (tab === false || !tab && !target.getClientRects().length) return null;
      return { target, tab };
    } catch { return null; }
  }
  function report(message) { state.message = message; text(bound?.querySelector('[data-activity-status]'), message); }
  function route(name, view) {
    if (!destination(name, view)) { report('Existing sample destination unavailable or mismatched; navigation refused.'); return; }
    bound.close();
    const action = {key:view.key, name, url:location.href, root:bound}; pending = action;
    requestAnimationFrame(() => {
      const current = synchronize();
      if (!current || pending !== action || current.key !== action.key || action.root !== bound || location.href !== action.url || !document.hasFocus()) return;
      const next = destination(name, current);
      if (!next) { pending = null; return; }
      if (next.tab) next.tab.click();
      requestAnimationFrame(() => {
        const current = synchronize();
        if (!current || pending !== action || current.key !== action.key || action.root !== bound || location.href !== action.url || !document.hasFocus()) return;
        const next = destination(name, current); pending = null;
        if (!next || !next.target.getClientRects().length || next.tab && next.tab.getAttribute('aria-selected') !== 'true') return;
        if (!next.target.hasAttribute('tabindex')) next.target.setAttribute('tabindex', '-1');
        location.hash = next.target.id; next.target.focus({preventScroll:true}); next.target.scrollIntoView();
      });
    });
  }
  function click(event) {
    const node = event.target instanceof Element && event.target.closest('[data-activity-action], [data-activity-filter], [data-notification-route]');
    if (!node || !bound?.contains(node)) return;
    event.preventDefault(); pending = null;
    const view = synchronize();
    if (!view || node.disabled || !node.isConnected || !bound.open || !node.getClientRects().length || !document.hasFocus()) return;
    if (node.dataset.notificationRoute) { route(node.dataset.notificationRoute, view); return; }
    const focused = document.activeElement === node;
    if (node.dataset.activityFilter) { state.filter = node.dataset.activityFilter; state.message = `${node.textContent} sample filter selected.`; }
    else if (node.dataset.activityAction === 'mark-read') {
      const id = node.closest('[data-activity-item]')?.dataset.activityItem;
      if (!source.some(record => record[0] === id) || state.read.includes(id) || state.cleared.includes(id)) return;
      state.read.push(id); state.message = 'Sample marked read locally. No approval or acknowledgement was sent.';
    } else if (node.dataset.activityAction === 'clear-read') {
      state.cleared = [...new Set([...state.cleared, ...state.read])]; state.message = 'Read samples cleared locally. No persisted record was deleted.';
    } else return;
    apply(bound, view);
    if (focused && (node.disabled || !node.getClientRects().length) && document.activeElement === node)
      bound.querySelector(`[data-activity-filter="${state.filter}"]`).focus({preventScroll:true});
  }
  function schedule() { if (!queued && !disposed) { queued = true; queueMicrotask(() => { queued = false; synchronize(); }); } }
  function cancel(event) { if (event.isTrusted && event.target?.isConnected) pending = null; }
  function cancelWindow() { pending = null; }
  const observer = new MutationObserver(schedule);
  observer.observe(document.body, {subtree:true, childList:true, characterData:true, attributes:true,
    attributeFilter:['data-notification-source','data-activity-item','data-activity-severity','data-sample-context-requested','data-sample-context-displayed','data-fragment-navigation','aria-busy','hidden','inert','aria-hidden']});
  for (const name of ['pointerdown','keydown','focusin']) document.addEventListener(name, cancel, true);
  for (const name of ['hashchange','popstate','blur']) window.addEventListener(name, cancelWindow);
  window.oyatieSampleNotifications = {
    snapshot() { const view = synchronize(); return view && state?.key === view.key ? {...state, read:[...state.read], cleared:[...state.cleared]} : null; },
    dispose() { disposed = true; pending = null; observer.disconnect(); bound?.removeEventListener('click', click);
      for (const name of ['pointerdown','keydown','focusin']) document.removeEventListener(name, cancel, true);
      for (const name of ['hashchange','popstate','blur']) window.removeEventListener(name, cancelWindow); }
  };
  synchronize();
})();
"##;

#[cfg(test)]
#[path = "notifications_test.rs"]
mod tests;
