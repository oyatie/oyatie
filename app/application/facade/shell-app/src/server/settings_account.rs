pub(super) const SCRIPT: &str = r##"
(() => {
  const host = document.getElementById('dashboard-island-root');
  const all = (root, selector) => [...(root?.querySelectorAll(selector) || [])];
  const unique = (root, selector) => { const nodes = all(root, selector); return nodes.length === 1 ? nodes[0] : null; };
  const setText = (node, value) => { if (node && node.textContent !== value) node.textContent = value; };
  const setAttr = (node, name, value) => { if (node?.getAttribute(name) !== String(value)) node?.setAttribute(name, String(value)); };
  const kinds = ['auth', 'sessions', 'roles', 'org', 'onboarding'];
  const setupActions = ['advance-onboarding','advance-setup','back-onboarding','reset-onboarding'];
  const accountControls = '[data-identity-action], [data-identity-route-action], [data-identity-anchor-action], [data-onboarding-anchor-action], [data-account-route], .identity-settings-rail [data-identity-tab]';
  const stepIDs = ['welcome','legal-profile','bank-payment','payroll-calendar','insurance','jurisdiction','people-import','retention-policy','review'];
  const destinations = { workflow: 'workflow-studio', finance: 'payroll-cockpit', payroll: 'payroll-cockpit', policy: 'cockpit-panel-policy', evidence: 'evidence-spine', audit: 'resource-panel-audit', cloud: 'cockpit-panel-topology', tasks: 'tasks-title', schedule: 'schedule-title', catalog: 'service-catalog', mail: 'work-hub', community: 'work-hub' };
  const labels = { en: {title:'Settings · sample preview',profile:'Sample profile',appearance:'Drawer appearance',integrations:'Sample integrations',audit:'Sample audit examples'}, ko: {title:'설정 · 샘플 미리보기',profile:'샘플 프로필',appearance:'서랍 표시',integrations:'샘플 연결',audit:'샘플 감사 예시'} };
  const bound = new WeakMap();
  let current = null, pending = null, queued = false, density = 'comfortable', locale = 'en', ownHashChange = null, assigningHash = null;
  const owner = () => { const root = unique(document, '#identity-workforce-service'); return unique(document, '#dashboard-island-root') === host && host?.contains(root) ? root : null; };
  const drawer = () => unique(document, 'dialog#settings-center');
  const sameView = (a, b) => a?.isConnected && b?.isConnected && host?.contains(a) && host.contains(b)
    && a.closest('[data-island="render-envelope-dashboard"]') === b.closest('[data-island="render-envelope-dashboard"]');
  const visible = node => node?.isConnected && !node.closest('[hidden], [inert], [aria-hidden="true"]') && node.getClientRects().length > 0;
  const knownRoute = route => Object.hasOwn(destinations, route) || kinds.includes(route) || route === 'employees';
  function controlIntent(control, root = owner()) {
    if (!root?.contains(control)) {
      const settings = drawer(), preference = control.dataset.settingsAction;
      if (!settings?.contains(control)) return null;
      if (['density-comfortable','density-compact','locale-ko','locale-en'].includes(preference)) return {preference,origin:settings};
      const route = preference === 'open-identity' || control.dataset.utilityRoute === 'identity' ? 'auth' : control.dataset.utilityRoute;
      return knownRoute(route) ? {route,origin:settings} : null;
    }
    if (control.closest('[data-identity-panel="employees"]')) return null;
    const rail = control.closest('.identity-settings-rail');
    if (rail) return unique(root, '.identity-settings-rail') === rail && control.tagName === 'BUTTON' && kinds.includes(control.dataset.identityTab)
      ? {route:control.dataset.identityTab,origin:root} : null;
    const facts = control.closest('[data-account-facts]');
    const header = '.identity-service-head .identity-service-actions button[data-identity-action="open-audit"][data-account-focus="audit"]';
    if (!(facts && kinds.includes(facts.dataset.accountFacts) && unique(root, `[data-account-facts="${facts.dataset.accountFacts}"]`) === facts)
      && unique(root, header) !== control) return null;
    const action = control.dataset.identityAction || control.dataset.identityAnchorAction || control.dataset.onboardingAnchorAction;
    if (['add-passkey','review-roles',...setupActions].includes(action)) return {action,origin:root};
    const route = control.dataset.identityRouteAction || control.dataset.accountRoute
      || (action === 'open-audit' ? 'audit' : action === 'import-employees' ? 'employees' : action?.startsWith('route-') ? action.slice(6) : null);
    return knownRoute(route) ? {route,origin:root} : null;
  }
  function applyAvailability(root, available) {
    all(root, accountControls).filter(node => controlIntent(node, root)?.origin === root).forEach(node => {
      if (node.tagName === 'A') setAttr(node, 'aria-disabled', !available);
      else if (node.disabled !== !available) node.disabled = !available;
    });
  }
  function signature(node) {
    if (!node) return null;
    const copy = node.cloneNode(true);
    // Empty renderer comments are metadata, not authored sample facts.
    const walker = document.createTreeWalker(copy, NodeFilter.SHOW_COMMENT), comments = [];
    for (let comment = walker.nextNode(); comment; comment = walker.nextNode()) if (comment.data === '') comments.push(comment);
    comments.forEach(comment => comment.remove());
    all(copy, '[data-account-local]').forEach(item => {
      item.textContent = ''; item.removeAttribute('style'); item.removeAttribute('aria-valuenow'); item.removeAttribute('aria-valuetext');
    });
    all(copy, '[data-setup-step]').forEach(item => { item.removeAttribute('class'); item.removeAttribute('aria-current'); });
    all(copy, 'button').forEach(item => item.removeAttribute('disabled'));
    all(copy, '[data-account-route]').forEach(item => item.removeAttribute('aria-disabled'));
    return copy.innerHTML;
  }
  function htmlSignature(html) { const node = document.createElement('div'); node.innerHTML = html; return signature(node); }
  function validate(root) {
    try {
      if (!root || !['ssr','island'].includes(root.dataset.accountMode)) return null;
      const key = root.dataset.accountSource, source = JSON.parse(key), sample = source[3];
      if (bound.has(root) && bound.get(root) !== key || !Array.isArray(source) || source.length !== 4
        || source.slice(0,3).some(item => typeof item !== 'string' || !item)
        || !['tenant-admin','corporate-office','healthcare-clinician'].includes(source[0]) || source[0] !== root.dataset.accountContext || source[1] !== root.dataset.accountLabel || source[2] !== root.dataset.accountHubSource
        || !Array.isArray(sample) || sample.length !== 3 || typeof sample[0] !== 'string' || typeof sample[1] !== 'string'
        || !Array.isArray(sample[2]) || sample[2].length !== kinds.length
        || signature(unique(root, '[data-account-person]')) !== htmlSignature(sample[0])
        || signature(unique(root, '[data-account-context-facts]')) !== htmlSignature(sample[1])) return null;
      const panels = all(root, '[data-account-facts]');
      if (panels.length !== kinds.length) return null;
      for (let index = 0; index < kinds.length; index++) {
        const pair = sample[2][index], panel = unique(root, `[data-account-facts="${kinds[index]}"]`);
        if (!Array.isArray(pair) || pair.length !== 2 || pair[0] !== kinds[index] || typeof pair[1] !== 'string'
          || htmlSignature(panel?.outerHTML || '') !== htmlSignature(pair[1])) return null;
      }
      const steps = all(root, '[data-setup-step]'), focus = all(root, '[data-account-focus]');
      if (steps.length !== stepIDs.length || steps.some((step,index) => step.dataset.setupStep !== stepIDs[index])
        || new Set(focus.map(node => node.dataset.accountFocus)).size !== focus.length
        || ['status','reset'].some(name => all(root, `[data-${name === 'status' ? 'identity-status' : 'account-reset'}]`).length !== 1)) return null;
      const dashboard = root.closest('[data-island="render-envelope-dashboard"]');
      const picker = unique(dashboard, '[data-sample-context-requested]');
      const hub = unique(document, '#work-hub'), workspace = unique(hub, '.communications-hub[data-hub-source]');
      const available = sameView(root, hub) && workspace?.dataset.hubSource === source[2] && workspace.dataset.hubContext === source[0]
        && workspace.dataset.hubMode === root.dataset.accountMode && root.dataset.accountAvailable === 'true' && dashboard?.getAttribute('aria-busy') === 'false'
        && dashboard.dataset.fragmentNavigation === root.dataset.accountMode && !dashboard.querySelector('.fetch-error')
        && picker?.dataset.sampleContextRequested === source[0] && picker.dataset.sampleContextDisplayed === source[0];
      if (!bound.has(root)) all(root, '[data-account-route]').filter(anchor => controlIntent(anchor, root)).forEach(anchor => anchor.addEventListener('click', event => {
        if (!anchor.isConnected || root !== owner() || !anchor.getClientRects().length || !validate(root)?.available) event.preventDefault();
      }));
      bound.set(root, key);
      return { root, key, source, available, mode: root.dataset.accountMode };
    } catch { return null; }
  }
  function tabTarget(panel) {
    const tab = unique(document, `#${panel?.getAttribute('aria-labelledby')}`), list = tab?.closest('[role="tablist"]');
    const tabs = all(list, '[role="tab"]');
    if (!tabs.length || tabs.filter(node => node.getAttribute('aria-selected') === 'true').length !== 1
      || tabs.some(node => { const controlled = unique(document, '#' + node.getAttribute('aria-controls'));
        return node.parentElement !== list || unique(document, '#' + node.id) !== node
          || !controlled || controlled.getAttribute('role') !== 'tabpanel' || controlled.getAttribute('aria-labelledby') !== node.id
          || controlled.id !== node.id.replace('-tab-', '-panel-') || !sameView(current.root, controlled); })) return null;
    if (!panel || !tab || tab.disabled || tab.parentElement !== list || tab.getAttribute('role') !== 'tab'
      || tab.getAttribute('aria-controls') !== panel.id || unique(document, `#${panel.id}`) !== panel || !sameView(current.root, panel)) return null;
    return tab;
  }
  function accountPanel(kind) { return unique(current?.root, `[data-identity-panel="${kind}"]`); }
  function selectedKind() { return current?.root.querySelector('.identity-tabs [aria-selected="true"]')?.dataset.identityTab; }
  function reflectRail() {
    const selected = selectedKind();
    all(current?.root, '.identity-settings-rail [data-identity-tab]').forEach(node => {
      const active = node.dataset.identityTab === selected;
      node.classList.toggle('active', active);
      if (active) setAttr(node, 'aria-current', 'page'); else node.removeAttribute('aria-current');
    });
  }
  function applySetup() {
    const root = current.root, count = current.complete.filter(Boolean).length, percent = Math.floor(count * 100 / stepIDs.length);
    setText(root.querySelector('[data-account-local="count"]'), `${count} of 9 sample steps complete · ${percent}%`);
    const bar = root.querySelector('[data-account-local="progress"]');
    setAttr(bar, 'aria-valuenow', percent); setAttr(bar, 'aria-valuetext', `${count} of 9 sample steps complete`);
    if (bar.style.getPropertyValue('--bar') !== `${percent}%`) bar.style.setProperty('--bar', `${percent}%`);
    all(root, '[data-setup-step]').forEach((step, index) => {
      step.classList.toggle('done', current.complete[index]); step.classList.toggle('active', index === current.step);
      setAttr(step, 'aria-current', index === current.step ? 'step' : 'false');
      setText(step.querySelector('[data-account-local]'), current.complete[index] ? 'Sample complete' : 'Sample incomplete');
    });
    const step = all(root, '[data-setup-step]')[current.step];
    setText(root.querySelector('[data-account-local="current-step"]'), step.querySelector('span').textContent);
    const message = count === 9 ? 'All nine sample steps complete locally. No tenant admission, go-live, readiness, legal or bank verification occurred.'
      : 'Inspect or complete this illustrative sample step only. No import, approval, verification or admission occurs.';
    setText(root.querySelector('[data-account-local="setup-status"]'), message);
    setText(root.querySelector('[data-onboarding-anchor-status]'), message);
  }
  function apply() {
    if (!current) return;
    const root = current.root;
    applyAvailability(root, current.available);
    setText(root.querySelector('[data-identity-status]'), current.available ? current.message : 'Sample source loading, failed, missing, invalid or mismatched; account actions unavailable. Live authority Unknown.');
    const reset = root.querySelector('[data-account-reset]'); if (reset.hidden !== !current.reset) reset.hidden = !current.reset;
    applySetup(); reflectRail();
  }
  function preferences() {
    const root = drawer(); if (!root) return;
    setAttr(root, 'data-settings-density', density);
    all(root, '[data-settings-label]').forEach(node => {
      setText(node, labels[locale][node.dataset.settingsLabel]); setAttr(node, 'lang', locale);
    });
    all(root, '[data-settings-action]').filter(node => node.dataset.settingsAction !== 'open-identity').forEach(node => {
      setAttr(node, 'aria-pressed', ['density-' + density, 'locale-' + locale].includes(node.dataset.settingsAction));
    });
  }
  function restoreFocus(previous) {
    const saved = current.focus;
    if (!saved || saved.node.isConnected || !current.available || !document.hasFocus() || location.href !== saved.url) return;
    const active = document.activeElement;
    if (active && active !== document.body && active.isConnected) { current.focus = null; return; }
    if (previous.selected && saved.url === previous.selectionURL) {
      const tab = tabTarget(accountPanel(previous.selected)); if (tab) tab.click();
    }
    const target = all(current.root, '[data-account-focus]').find(node => node.dataset.accountFocus === saved.token)
      || unique(current.root, `#${saved.token}`);
    current.focus = null;
    if (target && !target.disabled && target.getClientRects().length) target.focus({preventScroll:true});
  }
  function synchronize() {
    const root = owner(), observed = validate(root), previous = current;
    if (!observed) {
      pending = null;
      if (previous) {
        previous.available = false;
        // A native remount briefly removes all Account roots from the stable host.
        if (!host?.isConnected || unique(document, '#dashboard-island-root') !== host
          || all(document, '#identity-workforce-service').length || previous.focus?.node.isConnected) previous.focus = null;
      }
      for (const refused of all(document, '#identity-workforce-service')) {
        applyAvailability(refused, false);
        setText(refused.querySelector('[data-identity-status]'), 'Account sample invalid or unavailable; actions refused.');
      }
      return;
    }
    const same = previous?.key === observed.key;
    current = { ...observed, complete: same ? previous.complete : stepIDs.map((_,index) => index < 3), step: same ? previous.step : 3,
      focus: same ? previous.focus : null, selected: same ? previous.selected : null, selectionURL: same ? previous.selectionURL : null,
      reset: same ? previous.reset : !!previous, message: same ? previous.message : 'Sample examples only · live authority Unknown / unavailable' };
    if (!same || !current.available) pending = null;
    apply(); preferences();
    if (same) restoreFocus(previous);
    if (same && previous.root !== root && pending) openRoute(pending);
  }
  function schedule() { if (!queued) { queued = true; queueMicrotask(() => { queued = false; synchronize(); }); } }
  function targetBinding(boundary, name) {
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
  document.addEventListener('click', event => {
    if (!(event.target instanceof Element) || !event.target.isConnected || event.defaultPrevented) return;
    const settings = drawer(), control = event.target.closest('[data-settings-action], [data-utility-route], ' + accountControls);
    if (!control || control.disabled || !control.getClientRects().length) return;
    const root = control.closest('#identity-workforce-service'), intent = controlIntent(control, root); if (!intent) return;
    if (root && intent.origin === root && (root !== owner() || !validate(root)?.available)) { event.preventDefault(); synchronize(); return; }
    if (event.button !== 0 || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    event.preventDefault(); pending = null;
    const {preference,action,route,origin} = intent;
    if (preference) {
      if (preference.startsWith('density-')) density = preference.slice(8); else locale = preference.slice(7);
      preferences(); setText(settings.querySelector('[data-settings-status]'), `Drawer ${density} spacing · ${locale} title and tab label preview. Page memory only; resets on reload.`); return;
    }
    synchronize(); if (!current?.available || !validate(current.root)?.available) { setText(settings?.querySelector('[data-settings-status]'), 'Current account sample unavailable; destination refused.'); return; }
    if (action === 'add-passkey') { passkeyEntry(); return; }
    if (action === 'review-roles') { report('These are sample grant rows. Live membership, grants and policy decisions are Unknown / unavailable; no grant changed.'); return; }
    if (setupActions.includes(action)) {
      if (action === 'back-onboarding') current.step = Math.max(0,current.step - 1);
      else if (action === 'reset-onboarding') { current.complete = stepIDs.map((_,index) => index < 3); current.step = 3; }
      else { current.complete[current.step] = true; current.step = Math.min(8,current.step + 1); }
      applySetup(); return;
    }
    pending = {key:current.key,route,url:location.href,origin}; openRoute(pending);
  });
  document.addEventListener('focusin', event => {
    if (!current || !(event.target instanceof Element)) return;
    const control = event.target.closest('[data-account-focus], .identity-tabs [role="tab"]');
    if (current.root.contains(control) && !control.closest('[data-identity-panel="employees"]')) {
      current.focus = {node:control,token:control.dataset.accountFocus || control.id,url:location.href};
      current.selected = selectedKind(); current.selectionURL = location.href;
    } else if (event.target !== document.body && event.target.isConnected) { current.focus = null; if (pending && !pending.origin.contains(event.target)) pending = null; }
  });
  for (const name of ['pointerdown','keydown']) document.addEventListener(name, event => {
    if (event.isTrusted) { pending = null; if (current) current.focus = null; }
  }, true);
  function cancelIntent() { pending = null; if (current) { current.focus = null; current.selectionURL = null; } }
  window.addEventListener('hashchange', event => {
    const own = ownHashChange && event.oldURL === ownHashChange.oldURL && event.newURL === ownHashChange.newURL && location.href === event.newURL;
    ownHashChange = null; if (!own) cancelIntent();
  });
  window.addEventListener('popstate', event => {
    if (assigningHash && event.isTrusted && !assigningHash.popstateSeen && location.href === assigningHash.newURL) { assigningHash.popstateSeen = true; return; }
    ownHashChange = null; cancelIntent();
  });
  window.addEventListener('blur', cancelIntent);
  if (host) new MutationObserver(schedule).observe(host, {childList:true,subtree:true,attributes:true,attributeFilter:['aria-selected','aria-busy','data-account-source','data-account-available']});
  synchronize(); preferences();
})();
"##;

#[cfg(test)]
#[path = "settings_account_test.rs"]
mod tests;
