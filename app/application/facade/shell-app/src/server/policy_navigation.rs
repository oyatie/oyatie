pub(super) const SCRIPT: &str = r#"
(() => {
  let pending = null;
  let fragmentDestination = null;
  const panel = () => document.getElementById('cockpit-panel-policy');
  function reveal(target, focus) {
    const owner = target.closest('[role="tabpanel"]');
    const tab = owner && document.getElementById(owner.getAttribute('aria-labelledby'));
    if (owner && tab?.getAttribute('aria-controls') === owner.id) {
      for (const candidate of tab.closest('[role="tablist"]').querySelectorAll('[role="tab"]')) {
        const selected = candidate === tab;
        candidate.setAttribute('aria-selected', String(selected));
        candidate.setAttribute('tabindex', selected ? '0' : '-1');
        candidate.classList.toggle('active', selected);
        document.getElementById(candidate.getAttribute('aria-controls'))?.classList.toggle('active', selected);
      }
    }
    const detail = target.closest('details');
    if (detail) detail.open = true;
    if (focus) {
      const destination = target instanceof HTMLDetailsElement ? target.querySelector('summary') : target;
      if (!destination.hasAttribute('tabindex')) destination.setAttribute('tabindex', '-1');
      destination.focus({ preventScroll: true });
      target.scrollIntoView({ block: 'start' });
    }
  }
  function chooseChannel(current, channel) {
    const hub = document.getElementById('work-hub');
    const board = hub?.querySelector('[data-comms-product-board]');
    if (!hub || !board) return false;
    if (current.dataset.policyMode === 'island') {
      if (board.dataset.commsBoardSurface !== channel) {
        [...hub.querySelectorAll('.hub-tabs [role="tab"]')]
          .find(tab => tab.textContent.trim() === channel)?.click();
      }
    } else {
      const template = current.querySelector(`[data-policy-hub-template="${channel}"]`);
      const workspace = hub.querySelector('.communications-hub');
      if (!workspace || !template?.content.firstElementChild) return false;
      workspace.replaceWith(template.content.firstElementChild.cloneNode(true));
    }
    return true;
  }
  function apply(current, action, focus) {
    const target = document.getElementById(action.target);
    if (!target || (action.channel && !chooseChannel(current, action.channel))) return false;
    const status = current.querySelector('[data-policy-anchor-status]');
    if (status) status.textContent = action.channel
      ? `${action.channel} sample preview opened locally; no message sent or posted.`
      : target.closest('details')
        ? `${target.textContent}: ${target.closest('details').querySelector('p').textContent}`
        : `${action.label} sample view opened locally; no permission or persisted evidence was verified.`;
    reveal(target, focus);
    action.destination = target;
    // Replay ends once the island owns the requested route; later user choices win.
    if (current.dataset.policyMode === 'island' && current.isConnected && target.isConnected
        && (!action.channel || document.querySelector('#work-hub [data-comms-product-board]')?.dataset.commsBoardSurface === action.channel)) pending = null;
    return true;
  }
  document.addEventListener('click', (event) => {
    if (!(event.target instanceof Element) || event.defaultPrevented || event.button !== 0
        || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    const current = panel();
    const control = event.target.closest('[data-policy-anchor-action][data-policy-target]');
    if (!current || !control || !current.contains(control)) return;
    const action = {
      context: current.dataset.policyContext,
      source: current,
      target: control.dataset.policyTarget,
      channel: control.dataset.policyChannel,
      label: control.textContent.trim(),
      focus: true,
    };
    pending = action;
    if (!apply(current, action, true)) { pending = null; return; }
    if (location.hash !== `#${action.target}`) location.hash = action.target;
    fragmentDestination = action.destination;
    // Expanded explanations belong to the current DOM, not a later island mount.
    if (action.destination.closest('details')) pending = null;
  });
  document.addEventListener('focusin', (event) => {
    if (pending?.destination && !pending.destination.contains(event.target)
        && !pending.source.contains(event.target)) pending.focus = false;
  });
  function restore() {
    const current = panel();
    if (!current) return;
    if (pending && (pending.context !== current.dataset.policyContext
        || location.hash !== `#${pending.target}`)) pending = null;
    if (pending && pending.source !== current) {
      const action = pending;
      const focus = action.focus && (document.activeElement === document.body
        || !action.destination?.isConnected || action.destination.contains(document.activeElement));
      action.source = current;
      apply(current, action, focus);
    }
    let target;
    try { target = document.getElementById(decodeURIComponent(location.hash.slice(1))); }
    catch { fragmentDestination = null; return; }
    const detail = target?.closest('details');
    const previousDetail = fragmentDestination?.closest('details');
    const previousPanel = fragmentDestination?.closest('[data-policy-context]');
    // Matching initial takeover consumes the old native toggle once; later island replacements reset.
    if (current.dataset.policyMode === 'island' && previousPanel?.dataset.policyMode === 'ssr'
        && previousPanel !== current) {
      if (previousPanel.dataset.policyContext === current.dataset.policyContext
          && current.contains(target) && detail && previousDetail
          && detail.textContent === previousDetail.textContent) detail.open = previousDetail.open;
      fragmentDestination = target;
    }
    if (current.dataset.policyMode === 'ssr') {
      // Reveal a changed route or replacement DOM once; native details toggles own the same route.
      if (target !== fragmentDestination) {
        fragmentDestination = target;
        if (target) reveal(target, false);
      }
    }
  }
  // One document listener survives island replacement; it never binds callbacks to old nodes.
  const host = document.getElementById('dashboard-island-root');
  if (host) new MutationObserver(restore).observe(host, { childList: true, subtree: true });
  for (const event of ['hashchange', 'popstate']) window.addEventListener(event, restore);
  restore();
})();
"#;
