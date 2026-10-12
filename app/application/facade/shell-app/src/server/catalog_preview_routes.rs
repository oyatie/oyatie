pub(super) fn script() -> String {
    let source = crate::shell_capability_registry::production_shell_contract_source();
    let admitted_ids: std::collections::BTreeMap<_, _> =
        crate::render_envelope::OperatorContext::ALL
            .into_iter()
            .map(|context| {
                let granted = source.granted_actions(context);
                let ids: Vec<_> = source
                    .entries()
                    .iter()
                    .filter(|entry| granted.contains(entry.required_action.as_str()))
                    .map(|entry| entry.capability_id.as_str())
                    .collect();
                (context.id(), ids)
            })
            .collect();
    format!(
        "  const admittedIds = {};\n{SCRIPT}",
        serde_json::to_string(&admitted_ids).expect("native Catalog admitted identities serialize")
    )
}

pub(super) const SCRIPT: &str = r##"
  const cardRoutes = {
    'tenant-admin':['identity-panel-auth',''], 'cloud-compute':['resource-panel-inventory',''],
    'cloud-network':['cockpit-panel-topology',''], finops:['cockpit-panel-finops',''],
    'audit-chain':['resource-panel-audit',''], 'work-home':['daily-execution',''], mail:['work-hub','Mail'],
    accounting:['finance-panel-ledger',''], 'human-resources':['identity-panel-employees',''],
    approvals:['command-center-workbench',''], 'workflow-studio':['workflow-studio',''],
    'foundry-ontology':['ontology-command-console',''], 'clinical-home':['',''],
    'patient-schedule':['',''], 'care-workflows':['',''], 'secure-messenger':['','']
  };
  const connectedRoutes = {
    'graph-workflow':['workflow-studio',''], 'graph-mail':['work-hub','Mail'],
    'graph-community':['work-hub','Community'], 'graph-evidence':['evidence-spine',''],
    workflow:['workflow-studio',''], cloud:['cockpit-panel-topology',''],
    gates:['resource-panel-gates',''], policy:['cockpit-panel-policy',''], audit:['resource-panel-audit',''],
    finance:['finance-panel-ledger',''], identity:['identity-panel-auth',''],
    daily:['daily-execution',''], evidence:['evidence-spine','']
  };
  const knownIdentity = id => Object.hasOwn(cardRoutes, id);
  const cardRoute = id => cardRoutes[id];
  function validCardRoute(row, id, values) {
    const anchors = all(row, '[data-catalog-route]'), reserved = all(row, '[data-catalog-action]');
    const [destination, channel] = cardRoute(id);
    if (id === 'foundry-ontology') return anchors.length === 0 && reserved.length === 1
      && reserved[0].tagName === 'BUTTON' && reserved[0].dataset.catalogAction === 'open'
      && reserved[0].dataset.catalogTarget === destination;
    if (reserved.length) return false;
    if (!destination) return anchors.length === 0;
    return anchors.length === 1 && anchors[0].tagName === 'A' && anchors[0].dataset.catalogRoute === id
      && anchors[0].getAttribute('href') === '#' + destination && anchors[0].dataset.catalogChannel === channel
      && anchors[0].querySelector('[data-catalog-field="Action"]')?.textContent === values.get('Action');
  }
  function connectedReason(root, anchor) {
    const key = anchor.dataset.catalogLink;
    if (!Object.hasOwn(connectedRoutes, key)) return null;
    const id = 'catalog-link-unavailable-' + key, reasons = all(root, `[data-catalog-link-reason="${key}"]`);
    const reason = reasons[0];
    return reasons.length === 1 && reason.tagName === 'SMALL' && reason.id === id
      && reason.classList.contains('catalog-open-reason') && anchor.nextElementSibling === reason
      && anchor.parentElement === reason.parentElement && reason.parentElement.classList.contains('catalog-connected-link')
      && all(document, '#' + id).length === 1 && document.getElementById(id) === reason
      && all(document, '#catalog-availability').length === 1 && root.querySelector('#catalog-availability')?.dataset.catalogAvailability === 'true'
      && ['catalog-availability', 'catalog-availability ' + id].includes(anchor.getAttribute('aria-describedby')) ? reason : null;
  }
  function validConnectedLinks(root) {
    const links = all(root, '[data-catalog-link]'), keys = links.map(node => node.dataset.catalogLink);
    return links.length === Object.keys(connectedRoutes).length && new Set(keys).size === links.length
      && all(root, '[data-catalog-link-reason]').length === links.length
      && links.every(node => {
        const route = connectedRoutes[node.dataset.catalogLink];
        return route && node.tagName === 'A' && node.getAttribute('href') === '#' + route[0]
          && node.dataset.catalogChannel === route[1] && connectedReason(root, node) !== null;
      });
  }
  function routeIdentity(anchor) {
    return anchor.hasAttribute('data-catalog-route') ? current?.records.has(anchor.dataset.catalogRoute)
      && cardRoute(anchor.dataset.catalogRoute) : connectedRoutes[anchor.dataset.catalogLink];
  }
  function nativeTab(destination) {
    const panel = destination.closest('[role="tabpanel"]');
    if (!panel) return {panel:null, tab:null};
    const tab = document.getElementById(panel.getAttribute('aria-labelledby'));
    const group = tab?.closest('[role="tablist"]');
    if (!tab || !group || !host.contains(tab) || !host.contains(group) || tab.disabled || !visible(tab) || tab.getAttribute('role') !== 'tab'
      || tab.getAttribute('aria-controls') !== panel.id || tab.parentElement !== group
      || group.hasAttribute('data-native-hub-tabs') || group.classList.contains('hub-tabs')
      || all(host, '#' + panel.id).length !== 1 || all(host, '#' + tab.id).length !== 1
      || all(group, `[aria-controls="${panel.id}"]`).length !== 1) return null;
    return {panel, tab};
  }
  function destinationFor(anchor) {
    const route = routeIdentity(anchor);
    if (!route || anchor.getAttribute('href') !== '#' + route[0] || anchor.dataset.catalogChannel !== route[1]) return null;
    const destination = document.getElementById(route[0]);
    return destination?.isConnected && host.contains(destination) && all(host, '#' + route[0]).length === 1
      && nativeTab(destination) ? destination : null;
  }
  function workspace() {
    const hub = document.getElementById('work-hub');
    const roots = hub ? all(hub, '.communications-hub[data-hub-source]') : [];
    if (!current?.available || !hub || !host.contains(hub) || all(host, '#work-hub').length !== 1 || roots.length !== 1) return null;
    const root = roots[0];
    if (root.dataset.hubContext !== current.source[0] || root.dataset.hubSource !== current.source[2]
      || root.dataset.hubMode !== current.mode || !visible(root)) return null;
    return root;
  }
  function channelStructure(root) {
    return root && all(root, '.hub-list').length === 1 && all(root, '.hub-detail').length === 1
      && all(root, '.hub-composer textarea').length === 1 && all(root, '[data-comms-product-board]').length === 1;
  }
  function channelReady(root, channel) {
    const tab = root?.querySelector('#hub-tab-' + channel.toLowerCase());
    const panel = root?.querySelector('.hub-channel-panel');
    return channelStructure(root) && tab?.getAttribute('aria-selected') === 'true'
      && tab.getAttribute('aria-controls') === panel?.id && panel?.getAttribute('aria-labelledby') === tab.id
      && root.querySelector('[data-comms-product-board]').dataset.commsBoardSurface === channel;
  }
  function reportOwned(token) {
    return report === token && current?.available && current.key === token.key && current.mode === token.mode
      && current.source[2] === token.hubSource;
  }
  function settleReport() {
    const token = report;
    if (!token || !reportOwned(token)) { report = null; return; }
    if (performance.now() >= token.expiresAt) {
      text(current.root.querySelector('[data-catalog-route-status]'), `${token.channel} sample outcome unavailable; no output created.`);
      report = null; return;
    }
    if (location.hash === token.hash && channelReady(workspace(), token.channel)) {
      text(current.root.querySelector('[data-catalog-route-status]'), `${token.channel} existing sample opened; no draft, message or receipt created.`);
      report = null;
    }
  }
  function openChannel(channel) {
    const root = workspace();
    if (!['Mail','Community','Messenger'].includes(channel) || !channelStructure(root)) return false;
    const tabs = all(root, '#hub-tab-' + channel.toLowerCase());
    const tab = tabs[0], composer = root.querySelector('.hub-composer textarea');
    if (current.mode === 'island' && (tabs.length !== 1 || !visible(tab) || tab.disabled || !visible(composer) || composer.disabled
      || tab.getAttribute('role') !== 'tab' || !root.contains(tab) || !root.contains(composer))) return false;
    if (channelReady(root, channel)) return true;
    if (current.mode === 'ssr') {
      const templates = all(current.root, '[data-catalog-hub-template]').filter(node => node.dataset.catalogHubTemplate === channel);
      const replacement = templates.length === 1 && templates[0].content.childElementCount === 1
        ? templates[0].content.firstElementChild?.cloneNode(true) : null;
      if (!replacement || replacement.dataset.hubContext !== current.source[0] || replacement.dataset.hubSource !== current.source[2]
        || replacement.dataset.hubMode !== 'ssr' || !channelReady(replacement, channel)) return false;
      root.replaceWith(replacement);
      return true;
    }
    dispatching = true;
    try { tab.click(); } finally { dispatching = false; }
    return true;
  }
  function revealSSR(destination) {
    const relation = nativeTab(destination);
    if (!relation) return false;
    dispatching = true;
    try {
      if (relation.tab) {
        relation.tab.click();
        if (relation.tab.getAttribute('aria-selected') !== 'true' || !relation.panel.classList.contains('active')) return false;
      }
      if (document.hasFocus() && visible(destination)) {
        if (!destination.hasAttribute('tabindex')) destination.setAttribute('tabindex', '-1');
        destination.focus({preventScroll:true});
      }
    } finally { dispatching = false; }
    return true;
  }
  function route(anchor, event) {
    if (event.defaultPrevented) return;
    const destination = destinationFor(anchor), identity = routeIdentity(anchor);
    report = null;
    if (!destination || !identity || !current.available) { event.preventDefault(); return; }
    const [id, channel] = identity;
    if (channel && !openChannel(channel)) {
      event.preventDefault(); text(current.root.querySelector('[data-catalog-route-status]'), `${channel} sample unavailable; no output created.`); return;
    }
    if (current.mode === 'ssr' && !revealSSR(destination)) {
      event.preventDefault(); text(current.root.querySelector('[data-catalog-route-status]'), 'Existing sample destination unavailable.'); return;
    }
    if (!channel) {
      text(current.root.querySelector('[data-catalog-route-status]'), 'Opening an existing anonymous sample; no live operation.'); return;
    }
    const token = {key:current.key, mode:current.mode, hubSource:current.source[2], channel, destination:id, hash:'#' + id, expiresAt:performance.now() + 5000};
    report = token;
    text(current.root.querySelector('[data-catalog-route-status]'), `Opening ${channel} existing sample; no output created.`);
    requestAnimationFrame(() => { if (reportOwned(token)) settleReport(); });
    setTimeout(() => { if (reportOwned(token)) settleReport(); }, 5000);
  }
"##;
