pub(super) const SCRIPT: &str = r#"
(() => {
  const host = document.getElementById('dashboard-island-root');
  const channels = { messenger: 'Messenger', mail: 'Mail', community: 'Community' };
  let pendingRoute = null;
  let savedFocus = null;
  let ownHashChange = null;
  let assigningOwnHash = null;
  let settling = false;
  const panel = () => document.getElementById('workflow-studio');
  const binding = current => ({ source: current.dataset.workflowSource, context: current.dataset.workflowContext });
  const matches = (current, saved) => current?.isConnected && saved.source && saved.context
    && current.dataset.workflowSource === saved.source && current.dataset.workflowContext === saved.context;
  const sameView = (current, target) => host?.contains(current) && host.contains(target)
    && current.closest('[data-island="render-envelope-dashboard"]') === target.closest('[data-island="render-envelope-dashboard"]');
  function cancelIntent() {
    pendingRoute = null;
    savedFocus = null;
  }
  function status(current, message) {
    const output = current.querySelector('[data-workflow-output-status]');
    if (output && output.textContent !== message) output.textContent = message;
  }
  function channelReady(workspace, channel) {
    const tab = [...(workspace?.querySelectorAll('.hub-tabs [role="tab"]') || [])]
      .find(candidate => candidate.textContent.trim() === channel);
    const list = workspace?.querySelector('.hub-list');
    const items = [...(list?.querySelectorAll('.hub-item') || [])];
    const detail = workspace?.querySelector('.hub-detail');
    const heading = detail?.querySelector('.comms-detail-card .eyebrow');
    return !!(workspace && tab?.getAttribute('aria-selected') === 'true'
      && workspace.querySelectorAll('.hub-tabs [aria-selected="true"]').length === 1
      && workspace.querySelector('[data-comms-product-board]')?.dataset.commsBoardSurface === channel
      && list && detail?.querySelector('article') && workspace.querySelector('.hub-composer textarea')
      && (items.length ? items[0].classList.contains('active')
        && items.filter(item => item.classList.contains('active')).length === 1
        && heading?.textContent.trim() === channel
        && detail.querySelector('.comms-detail-card h4')?.textContent === items[0].querySelector('strong')?.textContent
        : !heading));
  }
  function chooseChannel(current, action, hub) {
    const channel = channels[action.route];
    const workspace = hub?.querySelector('.communications-hub');
    if (!workspace || !sameView(current, hub)) return false;
    if (current.dataset.workflowMode === 'island') {
      const tab = [...workspace.querySelectorAll('.hub-tabs [role="tab"]')]
        .find(candidate => candidate.textContent.trim() === channel);
      if (!tab) return false;
      // The native tab owns channel and item selection. Its click is only a trigger.
      tab.click();
      return true;
    }
    if (channelReady(workspace, channel)) return true;
    const policy = document.getElementById('cockpit-panel-policy');
    if (policy?.dataset.policyMode !== 'ssr' || policy.dataset.policyContext !== action.context
      || !sameView(current, policy)) return false;
    const template = [...policy.querySelectorAll('[data-policy-hub-template]')]
      .find(candidate => candidate.dataset.policyHubTemplate === channel);
    const replacement = template?.content.firstElementChild?.cloneNode(true);
    if (!replacement || !channelReady(replacement, channel)) return false;
    // The static export nests this existing article; the native dashboard keeps it beside the hub.
    const payroll = workspace.querySelector('#payroll-mail-preview');
    if (payroll) workspace.before(payroll);
    workspace.replaceWith(replacement);
    return true;
  }
  function reveal(target, focus) {
    if (!target.hasAttribute('tabindex')) target.setAttribute('tabindex', '-1');
    if (focus) target.focus({ preventScroll: true });
    target.scrollIntoView({ block: 'start' });
  }
  function finishRoute(current, action) {
    if (pendingRoute !== action || !matches(current, action)) return;
    if (location.href !== action.intentURL) { cancelIntent(); return; }
    const target = document.getElementById(action.route === 'evidence' ? 'evidence-spine' : 'work-hub');
    if (!target || !sameView(current, target)
      || (action.route !== 'evidence' && !channelReady(target.querySelector('.communications-hub'), channels[action.route]))) {
      status(current, `${channels[action.route] || 'Evidence'} destination unavailable in this view; no native channel was confirmed and no output was created.`);
      pendingRoute = null;
      return;
    }
    action.destination = target;
    if (location.hash !== `#${target.id}`) {
      const change = { oldURL: location.href, newURL: new URL(`#${target.id}`, location.href).href, popstateSeen: false };
      action.intentURL = change.newURL;
      ownHashChange = change;
      assigningOwnHash = change;
      // Native fragment assignment can dispatch its one popstate before returning.
      try { location.hash = target.id; }
      finally { assigningOwnHash = null; }
    }
    if (pendingRoute !== action || location.href !== action.intentURL) return;
    status(current, `${channels[action.route] || 'Evidence'} existing sample view opened; no draft, message, publication, or receipt created or verified.`);
    const active = document.activeElement;
    const focus = action.focus && (active === document.body || !active?.isConnected
      || action.origin.contains(active) || action.previousDestination?.contains(active)
      || (!action.previousOrigin?.isConnected && host?.contains(active)));
    reveal(target, focus);
    if (current.dataset.workflowMode === 'island') pendingRoute = null;
  }
  function openRoute(current, action) {
    if (!matches(current, action)) { pendingRoute = null; return; }
    if (location.href !== action.intentURL) { cancelIntent(); return; }
    action.previousOrigin = action.origin;
    action.origin = current;
    action.previousDestination = action.destination;
    const hub = document.getElementById('work-hub');
    if (action.route !== 'evidence' && !chooseChannel(current, action, hub)) {
      status(current, `${channels[action.route]} destination unavailable until the full native channel is present; no output was created.`);
      pendingRoute = null;
      return;
    }
    // Confirm the complete native consumer after its reactive state has settled.
    requestAnimationFrame(() => finishRoute(panel(), action));
  }
  function rememberFocus(target, focusTime = savedFocus?.focusTime) {
    const current = panel();
    if (!current?.contains(target)) return false;
    const control = target.closest('[data-workflow-focus]');
    savedFocus = { ...binding(current), intentURL: location.href, focusTime, node: target, token: control?.dataset.workflowFocus || 'panel' };
    return true;
  }
  function restoreFocus(current) {
    if (!savedFocus || savedFocus.node.isConnected) return;
    if (location.href !== savedFocus.intentURL) { cancelIntent(); return; }
    if (!matches(current, savedFocus)) { savedFocus = null; return; }
    const active = document.activeElement;
    if (active && active !== document.body && active.isConnected && !host?.contains(active)) {
      savedFocus = null;
      return;
    }
    const previous = savedFocus;
    const start = previous.node.selectionStart, end = previous.node.selectionEnd;
    let target = [...current.querySelectorAll('[data-workflow-focus]')]
      .find(candidate => candidate.dataset.workflowFocus === previous.token
        && !candidate.disabled && !candidate.closest('[hidden]'));
    target ||= current;
    target.focus({ preventScroll: true });
    if (start !== undefined && typeof target.setSelectionRange === 'function') {
      target.setSelectionRange(start, end);
    }
    rememberFocus(target);
  }
  function synchronize() {
    if (settling) return;
    settling = true;
    queueMicrotask(() => {
      settling = false;
      const current = panel();
      if (!current) return;
      if (pendingRoute && !matches(current, pendingRoute)) pendingRoute = null;
      if (pendingRoute && !pendingRoute.origin.isConnected) openRoute(current, pendingRoute);
      restoreFocus(current);
    });
  }
  document.addEventListener('click', event => {
    if (!(event.target instanceof Element) || event.defaultPrevented || event.button !== 0
      || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    const current = panel();
    const control = event.target.closest('[data-workflow-output-route]');
    if (!current?.contains(control) || control.disabled
      || !(Object.hasOwn(channels, control.dataset.workflowOutputRoute) || control.dataset.workflowOutputRoute === 'evidence')) return;
    savedFocus = null;
    pendingRoute = { ...binding(current), intentURL: location.href, route: control.dataset.workflowOutputRoute, origin: current, focus: true };
    openRoute(current, pendingRoute);
  });
  // Later real user intent cancels pending replay, including a new native hub channel choice.
  for (const name of ['pointerdown', 'keydown']) document.addEventListener(name, event => {
    if (!event.isTrusted || !(event.target instanceof Element)) return;
    pendingRoute = null;
    if (!savedFocus?.node.isConnected || !panel()?.contains(event.target)) savedFocus = null;
  }, true);
  document.addEventListener('focusin', event => {
    if (!(event.target instanceof Element)) return;
    if (savedFocus && (!savedFocus.node.isConnected && host?.contains(event.target))) return;
    if (rememberFocus(event.target, event.timeStamp)) return;
    if (event.target === document.body) return;
    if (pendingRoute && (!pendingRoute.origin.isConnected || !pendingRoute.previousOrigin?.isConnected)
      && host?.contains(event.target)) return;
    savedFocus = null;
    if (pendingRoute && !pendingRoute.destination?.contains(event.target)) pendingRoute.focus = false;
  });
  document.addEventListener('input', event => {
    if (event.target instanceof Element) rememberFocus(event.target);
  });
  window.addEventListener('hashchange', event => {
    const own = ownHashChange && event.oldURL === ownHashChange.oldURL
      && event.newURL === ownHashChange.newURL && location.href === event.newURL;
    ownHashChange = null;
    if (!own) {
      pendingRoute = null;
      const active = document.activeElement;
      const ownedFocus = savedFocus && (savedFocus.node.isConnected
        ? active === savedFocus.node && matches(panel(), savedFocus)
        : active === document.body || !active?.isConnected || host?.contains(active));
      const newerFocus = event.isTrusted && ownedFocus
        && Number.isFinite(event.timeStamp) && Number.isFinite(savedFocus.focusTime)
        && event.timeStamp < savedFocus.focusTime
        && event.newURL === savedFocus.intentURL && location.href === event.newURL;
      if (!newerFocus) savedFocus = null;
    }
  });
  window.addEventListener('popstate', event => {
    if (assigningOwnHash && assigningOwnHash === ownHashChange && event.isTrusted
      && !assigningOwnHash.popstateSeen && location.href === assigningOwnHash.newURL) {
      assigningOwnHash.popstateSeen = true;
      return;
    }
    ownHashChange = null;
    cancelIntent();
  });
  if (host) new MutationObserver(synchronize).observe(host, { childList: true, subtree: true });
})();
"#;

#[cfg(test)]
#[path = "workflow_navigation_test.rs"]
mod tests;
