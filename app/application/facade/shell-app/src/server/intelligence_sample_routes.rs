pub(super) const SCRIPT: &str = r##"  function nativeTab(destination) {
    const panel = destination.closest('[role="tabpanel"]');
    if (!panel) return visible(destination) && (destination.id === 'workflow-studio' && destination.classList.contains('workflow-panel')
      && destination.dataset.workflowContext === current.context && destination.dataset.workflowMode === current.mode
      && destination.dataset.workflowSource || destination.id === 'work-hub' && workspace()) ? {panel:null, tab:null} : null;
    const tab = document.getElementById(panel.getAttribute('aria-labelledby')), group = tab?.closest('[role="tablist"]');
    const relation = destination.id === 'identity-panel-employees' ? ['identity-workforce-service','identity-tabs']
      : destination.id === 'finance-panel-ledger' ? ['finance-commercial-service','finance-tabs'] : null;
    if (!relation || !tab || !group || !host.contains(tab) || !visible(tab) || tab.disabled
      || group.hasAttribute('data-native-hub-tabs') || group.classList.contains('hub-tabs') || !group.classList.contains(relation[1])
      || !destination.closest('#' + relation[0]) || all(host, '#' + relation[0]).length !== 1
      || group.closest('#' + relation[0]) !== destination.closest('#' + relation[0])
      || destination.id === 'identity-panel-employees' && destination.dataset.identityPanel !== 'employees'
      || destination.id === 'finance-panel-ledger' && destination.dataset.financePanel !== 'ledger'
      || tab.getAttribute('role') !== 'tab' || tab.parentElement !== group || tab.getAttribute('aria-controls') !== panel.id
      || all(document, '#' + panel.id).length !== 1 || all(document, '#' + tab.id).length !== 1
      || all(group, `[aria-controls="${panel.id}"]`).length !== 1) return null;
    return {panel, tab};
  }
  function destinationFor(anchor) {
    const record = current?.records.get(anchor.dataset.intelligenceRoute);
    if (!record || !current.available || current.dismissed.has(record.item.id) || record.card.hidden || !visible(anchor)
      || anchor.closest('[hidden]') || !record.card.contains(anchor) || anchor.getAttribute('href') !== '#' + record.item.destination
      || anchor.dataset.intelligenceChannel !== (record.item.channel || '')) return null;
    const targets = all(document, '#' + record.item.destination), target = targets[0];
    return targets.length === 1 && target?.isConnected && host.contains(target) && nativeTab(target)
      && (!record.item.channel || workspace()) ? target : null;
  }
  function workspace() {
    const hubs = all(host, '#work-hub'), roots = hubs.length === 1 ? all(hubs[0], '.communications-hub[data-hub-source]') : [];
    const root = roots[0];
    if (!(roots.length === 1 && visible(root) && root.dataset.hubContext === current.context
      && root.dataset.hubSource === current.source[2] && root.dataset.hubMode === current.mode
      && all(root, '.hub-list').length === 1 && all(root, '.hub-detail').length === 1
      && all(root, '.hub-composer textarea').length === 1 && all(root, '[data-comms-product-board]').length === 1
      && all(root, '[data-native-hub-tabs][role="tablist"]').length === 1 && all(root, '.hub-channel-panel[role="tabpanel"]').length === 1)) return null;
    const tabs = all(root.querySelector('[data-native-hub-tabs]'), '[role="tab"]');
    return tabs.length === 3 && new Set(tabs.map(tab => tab.id)).size === 3
      && tabs.filter(tab => tab.getAttribute('aria-selected') === 'true').length === 1
      && ['Mail','Messenger','Community'].every(channel => tabs.some(tab => tab.id === 'hub-tab-' + channel.toLowerCase()))
      && ['Mail','Messenger','Community'].some(channel => channelReady(root, channel)) ? root : null;
  }
  function channelReady(root, channel) {
    const tabs = root ? all(root, '#hub-tab-' + channel.toLowerCase()) : [], panels = root ? all(root, '.hub-channel-panel') : [];
    const tab = tabs[0], panel = panels[0];
    return tabs.length === 1 && panels.length === 1 && tab.parentElement === root.querySelector('[data-native-hub-tabs][role="tablist"]')
      && tab.getAttribute('role') === 'tab' && panel.getAttribute('role') === 'tabpanel' && tab.getAttribute('aria-selected') === 'true'
      && tab.getAttribute('aria-controls') === panel.id && panel.getAttribute('aria-labelledby') === tab.id
      && all(root, '.hub-list').length === 1 && all(root, '.hub-detail').length === 1
      && all(root, '.hub-composer textarea').length === 1 && all(root, '[data-comms-product-board]').length === 1
      && root.querySelector('[data-comms-product-board]').dataset.commsBoardSurface === channel;
  }
  function openChannel(channel) {
    if (!['Mail','Messenger','Community'].includes(channel)) return false;
    const root = workspace(); if (!root) return false;
    if (current.mode === 'island') {
      const tabs = all(root, '#hub-tab-' + channel.toLowerCase()), tab = tabs[0], composer = root.querySelector('.hub-composer textarea');
      if (tabs.length !== 1 || !visible(tab) || tab.disabled || !visible(composer) || composer.disabled
        || tab.getAttribute('role') !== 'tab' || tab.parentElement !== root.querySelector('[data-native-hub-tabs][role="tablist"]')) return false;
    }
    if (channelReady(root, channel)) return true;
    if (current.mode === 'ssr') {
      const templates = all(current.root, '[data-intelligence-hub-template]').filter(node => node.dataset.intelligenceHubTemplate === channel);
      const replacement = templates.length === 1 && templates[0].content.childElementCount === 1 ? templates[0].content.firstElementChild?.cloneNode(true) : null;
      if (!replacement || replacement.dataset.hubContext !== current.context || replacement.dataset.hubSource !== current.source[2]
        || replacement.dataset.hubMode !== 'ssr' || !channelReady(replacement, channel)) return false;
      root.replaceWith(replacement); return true;
    }
    const tabs = all(root, '#hub-tab-' + channel.toLowerCase()), tab = tabs[0], composer = root.querySelector('.hub-composer textarea');
    if (tabs.length !== 1 || !visible(tab) || tab.disabled || !visible(composer) || composer.disabled
      || tab.getAttribute('role') !== 'tab' || !tab.closest('[role="tablist"]')?.hasAttribute('data-native-hub-tabs')) return false;
    dispatching = true; try { tab.click(); } finally { dispatching = false; }
    return true;
  }
  function settleReport() {
    const pending = report;
    if (!pending || !current?.available || pending.key !== current.key) { report = null; return; }
    if (performance.now() >= pending.expires) {
      text(current.root.querySelector('[data-intelligence-route-status]'), 'Existing sample outcome unavailable; no action output created.'); report = null; return;
    }
    const target = document.getElementById(pending.destination);
    if (location.href === pending.url && visible(target) && (!pending.channel || channelReady(workspace(), pending.channel))) {
      text(current.root.querySelector('[data-intelligence-route-status]'), 'Existing sample opened; no draft, message, workflow, permission or receipt created.'); report = null;
    }
  }
"##;
