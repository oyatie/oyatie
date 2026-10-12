pub(super) const SCRIPT: &str = r##"  function targetBinding(boundary, name) {
    try {
      const source = JSON.parse(boundary.dataset[name + 'Source']), hub = JSON.parse(current.source[2]);
      const length = {workflow:6,daily:8,ae:7,cloud:5,catalog:4}[name];
      if (!Array.isArray(source) || source.length !== length || source[0] !== current.source[0]
        || boundary.dataset[name + 'Context'] !== current.source[0] || boundary.dataset[name + 'Mode'] !== current.mode) return false;
      if (['workflow','daily'].includes(name)) {
        if (JSON.stringify(source.slice(0,name === 'workflow' ? 5 : 6)) !== JSON.stringify(hub.slice(0,name === 'workflow' ? 5 : 6))) return false;
      } else if (source[1] !== current.source[1] || source[2] !== current.source[2]
        || boundary.dataset[name + 'HubSource'] !== current.source[2]) return false;
      if (name === 'ae') {
        const roots = ['#evidence-spine','#resource-audit-console'].map(selector => unique(document, selector));
        if (!roots.includes(boundary) || source[1] !== boundary.dataset.aeLabel || roots.some(root => !sameView(current.root, root)
          || ['Source','Context','Label','HubSource','Mode'].some(field => root.dataset['ae' + field] !== boundary.dataset['ae' + field])
          || root.dataset.aeSourceAvailable !== 'true' || root.dataset.aeAvailable !== 'true')) return false;
      }
      if (['daily','ae','catalog'].includes(name) && boundary.dataset[name + 'Available'] !== 'true') return false;
      if (name === 'catalog' && boundary.dataset.catalogAdmitted !== 'true') return false;
      return true;
    } catch { return false; }
  }
  function destination(route) {
    const id = kinds.includes(route) || route === 'employees' ? accountPanel(route)?.id : destinations[route];
    const target = id && unique(document, `#${id}`);
    if (!target || !sameView(current.root, target)) return null;
    const panel = target.closest('[role="tabpanel"]'), tab = panel && tabTarget(panel);
    if (target.closest('[hidden], [inert], [aria-hidden="true"]') || panel && (!tab || !visible(tab)) || !panel && !visible(target)) return null;
    if (['evidence','audit'].includes(route) && !targetBinding(target.closest('#evidence-spine, #resource-audit-console'), 'ae')) return null;
    const bindings = [['[data-workflow-source]','workflow'], ['[data-daily-source]','daily'], ['[data-ae-source]','ae'], ['[data-cloud-source]','cloud'], ['[data-catalog-source]','catalog']];
    for (const [selector,name] of bindings) {
      const boundary = target.closest(selector);
      if (boundary && !targetBinding(boundary, name)) return null;
    }
    if (route === 'policy' && (target.dataset.policyContext !== current.source[0] || target.dataset.policyMode !== current.mode
      || !targetBinding(unique(document, '#cockpit-panel-topology'), 'cloud'))) return null;
    if (route === 'payroll' || route === 'finance') { if (!target.classList.contains('payroll-card')) return null; }
    return target;
  }
  function workspace() {
    const hub = destination('mail'), root = unique(hub, '.communications-hub[data-hub-source]');
    return root?.dataset.hubSource === current.source[2] && root.dataset.hubContext === current.source[0] && root.dataset.hubMode === current.mode ? root : null;
  }
  function channelReady(root, channel) {
    if (!['Mail','Messenger','Community'].includes(channel)) return false;
    const tab = unique(root, `#hub-tab-${channel.toLowerCase()}`), panel = unique(root, '.hub-channel-panel');
    const list = unique(root, '.hub-list'), detail = unique(root, '.hub-detail'), items = all(list, '.hub-item');
    const active = items.filter(node => node.classList.contains('active')), card = detail?.querySelector('.comms-detail-card');
    return !!(tab?.getAttribute('aria-selected') === 'true' && all(root, '.hub-tabs [aria-selected="true"]').length === 1
      && tab.getAttribute('aria-controls') === panel?.id && panel.getAttribute('aria-labelledby') === tab.id
      && root.querySelector('[data-comms-product-board]')?.dataset.commsBoardSurface === channel && list && detail
      && root.querySelector('.hub-composer textarea') && (items.length ? active.length === 1 && card?.querySelector('.eyebrow')?.textContent.trim() === channel
        && card.querySelector('h4')?.textContent === active[0].querySelector('strong')?.textContent : !card));
  }
  function chooseChannel(channel) {
    const root = workspace(); if (!root) return false;
    if (!channelReady(root, root.querySelector('.hub-tabs [aria-selected="true"]')?.textContent.trim())) return false;
    if (channelReady(root, channel)) return true;
    if (current.mode === 'island') {
      const tab = unique(root, `#hub-tab-${channel.toLowerCase()}`); if (!tab || tab.disabled) return false; tab.click(); return true;
    }
    const template = unique(current.root, `[data-account-hub-template="${channel}"]`);
    const replacement = template?.content.firstElementChild?.cloneNode(true);
    if (!replacement || replacement.dataset.hubSource !== current.source[2] || replacement.dataset.hubContext !== current.source[0]
      || replacement.dataset.hubMode !== 'ssr' || !channelReady(replacement, channel)) return false;
    const payroll = root.querySelector('#payroll-mail-preview'); if (payroll) root.before(payroll);
    root.replaceWith(replacement); return true;
  }
  function report(message) { current.message = message; setText(current.root.querySelector('[data-identity-status]'), message); setText(drawer()?.querySelector('[data-settings-status]'), message); }
  function openRoute(action) {
    if (!current?.available || pending !== action || action.key !== current.key || location.href !== action.url) return;
    const target = destination(action.route), channel = {mail:'Mail',community:'Community'}[action.route];
    if (!target || channel && !chooseChannel(channel)) { pending = null; report('Existing sample destination unavailable or incomplete; no output created.'); return; }
    const panel = target.closest('[role="tabpanel"]'), tab = panel && tabTarget(panel);
    if (tab) {
      if (action.route === 'employees') current.root.querySelector('.identity-settings-rail [data-identity-tab="employees"]').click();
      else tab.click();
    }
    requestAnimationFrame(() => {
      const observed = validate(owner());
      if (pending !== action || observed?.key !== action.key || !observed.available || location.href !== action.url) return;
      if (destination(action.route) !== target || !visible(target) || channel && !channelReady(workspace(), channel)
        || panel && (!panel.classList.contains('active') || tab.getAttribute('aria-selected') !== 'true')) {
        pending = null; report('Existing sample destination unavailable; no full consumer confirmed.'); return;
      }
      pending = null;
      report(action.route === 'employees' ? 'HR sample directory opened. Import is unavailable; no employee added or imported.'
        : `${channel || action.route} existing sample opened; no draft, message, publication, receipt or mutation created.`);
      const active = document.activeElement;
      const focus = document.hasFocus() && (!active || active === document.body || !active.isConnected || action.origin.contains(active) || active === tab);
      const settings = drawer(); if (settings?.open) settings.close();
      if (location.hash !== '#' + target.id) {
        ownHashChange = {oldURL:location.href,newURL:new URL('#' + target.id,location.href).href,popstateSeen:false};
        assigningHash = ownHashChange;
        try { location.hash = target.id; } finally { assigningHash = null; }
      }
      if (!target.hasAttribute('tabindex')) target.setAttribute('tabindex','-1');
      if (focus) target.focus({preventScroll:true});
      target.scrollIntoView({block:'start'});
    });
  }
  function passkeyEntry() {
    const panel = unique(document, '#oyatie-sign-in[data-sign-in-panel="true"]'), form = unique(panel, '#passkey-form');
    const invitation = unique(form, '.sign-in-invitation'), summary = invitation?.querySelector('summary');
    if (!panel?.isConnected || host.contains(panel) || !invitation || !summary || panel.getAttribute('aria-busy') === 'true' || panel.querySelector('[aria-busy="true"]') || panel.contains(document.activeElement)) {
      report('Real passkey invitation unavailable or an IAM ceremony is active; no credential action performed.'); return;
    }
    invitation.open = true; summary.focus({preventScroll:true}); panel.scrollIntoView({block:'start'});
    report('Existing Oyatie IAM invitation opened. Use administrator-provided IDs and invitation there; no enrollment started.');
  }
"##;
