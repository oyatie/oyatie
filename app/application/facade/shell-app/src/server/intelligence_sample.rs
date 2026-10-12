#[path = "intelligence_sample_routes.rs"]
mod routes;

pub(super) fn script() -> String {
    use crate::render_envelope::{IntelligenceSampleId, OperatorContext};
    let identities: std::collections::BTreeMap<_, _> = OperatorContext::ALL
        .into_iter()
        .map(|context| {
            let samples: std::collections::BTreeMap<_, _> = IntelligenceSampleId::ALL
                .into_iter()
                .filter(|id| id.context() == context)
                .map(|id| {
                    let route = id.route();
                    (
                        id.id(),
                        (
                            route.map(|(destination, _)| destination),
                            route.and_then(|(_, channel)| channel),
                        ),
                    )
                })
                .collect();
            (context.id(), samples)
        })
        .collect();
    format!("{STATE}{}{EVENTS}", routes::SCRIPT).replace(
        "__SAMPLE_IDENTITIES__",
        &serde_json::to_string(&identities).expect("native sample identities serialize"),
    )
}

const STATE: &str = r##"
(() => {
  const host = document.getElementById('dashboard-island-root');
  if (!host) return;
  const identities = __SAMPLE_IDENTITIES__;
  const boundSources = new WeakMap();
  const all = (root, selector) => [...root.querySelectorAll(selector)];
  const visible = node => node?.isConnected && node.getClientRects().length > 0;
  const text = (node, value) => { if (node && node.textContent !== String(value)) node.textContent = String(value); };
  const attr = (node, name, value) => { if (node.getAttribute(name) !== String(value)) node.setAttribute(name, String(value)); };
  const disable = (node, value) => { if (node.disabled !== value) node.disabled = value; };
  const hide = (node, value) => { if (node.hidden !== value) node.hidden = value; };
  let current = null, handoff = null, queued = false, dispatching = false, report = null;
  function owner() {
    const roots = all(document, '#intelligence-command-console');
    return roots.length === 1 && host.contains(roots[0]) && all(host, '[data-intelligence-root]').length === 1 ? roots[0] : null;
  }
  function validate(root) {
    try {
      if (!root || root.dataset.intelligenceRoot !== 'true' || root.dataset.intelligenceAdmitted !== 'true') return null;
      const key = root.dataset.intelligenceSource, source = JSON.parse(key), context = source[0], mode = root.dataset.intelligenceMode;
      if (boundSources.has(root) && boundSources.get(root) !== key) return null;
      if (!Array.isArray(source) || source.length !== 7 || !Object.hasOwn(identities, context)
        || source.slice(0,3).some(value => typeof value !== 'string' || !value) || typeof source[4] !== 'string'
        || !source[5] || typeof source[5] !== 'object' || Array.isArray(source[5]) || typeof source[6] !== 'string'
        || Object.keys(source[5]).sort().join(',') !== 'explanation,healthcare_enabled,label'
        || typeof source[5].label !== 'string' || typeof source[5].explanation !== 'string' || typeof source[5].healthcare_enabled !== 'boolean'
        || source[1] !== root.dataset.intelligenceLabel || source[2] !== root.dataset.intelligenceHubSource
        || context !== root.dataset.intelligenceContext || !['ssr','island'].includes(mode)
        || !['true','false'].includes(root.dataset.intelligenceAvailable) || !Array.isArray(source[3])) return null;
      const cards = all(root, '[data-intelligence-card]'), records = new Map(), admitted = identities[context];
      if (cards.length !== source[3].length || cards.length !== Object.keys(admitted).length) return null;
      for (const item of source[3]) {
        if (!item || Object.keys(item).sort().join(',') !== 'body,channel,destination,guardrail,id,title'
          || !Object.hasOwn(admitted, item.id) || records.has(item.id)
          || ['title','body','guardrail'].some(field => typeof item[field] !== 'string')
          || item.destination !== admitted[item.id][0] || item.channel !== admitted[item.id][1]) return null;
        const matches = cards.filter(card => card.dataset.intelligenceCard === item.id), card = matches[0];
        if (matches.length !== 1 || JSON.stringify(JSON.parse(card.dataset.intelligenceRecord)) !== JSON.stringify(item)) return null;
        for (const field of ['title','body','guardrail']) {
          const fields = all(card, `[data-intelligence-field="${field}"]`);
          if (fields.length !== 1 || fields[0].textContent !== item[field]) return null;
        }
        const details = all(card, '[data-intelligence-detail]'), traces = all(card, '[data-intelligence-trace]');
        const dismiss = all(card, '[data-intelligence-dismiss]'), links = all(card, '[data-intelligence-route]');
        const reasons = all(card, '[data-intelligence-route-reason]');
        if (details.length !== 1 || details[0].tagName !== 'DETAILS' || details[0].dataset.intelligenceDetail !== item.id
          || traces.length !== 1 || traces[0].tagName !== 'DETAILS' || traces[0].dataset.intelligenceTrace !== item.id
          || !details[0].contains(traces[0]) || !details[0].querySelector(':scope > summary') || !traces[0].querySelector(':scope > summary')
          || all(card, '[data-intelligence-close]').length !== 2 || dismiss.length !== 1 || dismiss[0].dataset.intelligenceDismiss !== item.id
          || reasons.length !== 1 || reasons[0].dataset.intelligenceRouteReason !== item.id
          || reasons[0].id !== 'intelligence-route-reason-' + item.id || all(document, '#' + reasons[0].id).length !== 1) return null;
        const expectedFocus = ['detail:', 'trace:', 'close-detail:', 'close-trace:', 'dismiss:'].map(prefix => prefix + item.id);
        if (item.destination) expectedFocus.push('route:' + item.id);
        const focus = all(card, '[data-intelligence-focus]').map(node => node.dataset.intelligenceFocus);
        const closeControls = all(card, '[data-intelligence-close]');
        if (focus.length !== expectedFocus.length || expectedFocus.some(token => !focus.includes(token))
          || details[0].querySelector(':scope > summary').dataset.intelligenceFocus !== 'detail:' + item.id
          || traces[0].querySelector(':scope > summary').dataset.intelligenceFocus !== 'trace:' + item.id
          || dismiss[0].tagName !== 'BUTTON' || closeControls.some(node => node.tagName !== 'BUTTON'
            || ![details[0], traces[0]].includes(node.closest('details'))
            || node.dataset.intelligenceClose !== 'true' || node.dataset.intelligenceFocus !==
              (node.closest('details') === traces[0] ? 'close-trace:' : 'close-detail:') + item.id)) return null;
        if (item.destination ? links.length !== 1 || links[0].tagName !== 'A'
          || links[0].dataset.intelligenceRoute !== item.id || links[0].getAttribute('href') !== '#' + item.destination
          || links[0].dataset.intelligenceChannel !== (item.channel || '')
          || links[0].getAttribute('aria-describedby') !== 'intelligence-availability ' + reasons[0].id : links.length !== 0) return null;
        records.set(item.id, {item, card});
      }
      const tokens = all(root, '[data-intelligence-focus]').map(node => node.dataset.intelligenceFocus);
      if (new Set(tokens).size !== tokens.length || tokens.length !== cards.length * 5 + cards.filter(card => card.querySelector('[data-intelligence-route]')).length + 2
        || ['status','remaining','total','empty','source-reset','reset','availability','route-status'].some(name => all(root, `[data-intelligence-${name}]`).length !== 1)
        || all(root, '#intelligence-title').length !== 1 || all(document, '#intelligence-title').length !== 1
        || all(root, '#intelligence-live-unavailable').length !== 1 || all(document, '#intelligence-live-unavailable').length !== 1) return null;
      if (root.querySelector('[data-intelligence-reset]').tagName !== 'BUTTON'
        || root.querySelector('[data-intelligence-reset]').dataset.intelligenceFocus !== 'reset'
        || root.querySelector('#intelligence-title').dataset.intelligenceFocus !== 'heading'
        || all(root, '[data-intelligence-unavailable]').length !== cards.length + 1
        || all(root, '[data-intelligence-unavailable]').some(node => node.tagName !== 'BUTTON' || !node.disabled
          || node.getAttribute('aria-describedby') !== 'intelligence-live-unavailable')) return null;
      const dashboard = root.closest('[data-island="render-envelope-dashboard"]'), pickers = dashboard ? all(dashboard, '[data-sample-context-requested]') : [];
      const available = root.dataset.intelligenceAvailable === 'true' && dashboard?.getAttribute('aria-busy') === 'false'
        && dashboard.dataset.fragmentNavigation === mode && pickers.length === 1
        && pickers[0].dataset.sampleContextRequested === context && pickers[0].dataset.sampleContextDisplayed === context && visible(root);
      if (!boundSources.has(root)) {
        all(root, '[data-intelligence-route]').forEach(anchor => {
          anchor.addEventListener('click', event => {
            if (!anchor.isConnected) event.preventDefault();
          });
        });
      }
      boundSources.set(root, key);
      return {root, key, source, context, mode, records, available};
    } catch { return null; }
  }
  function controls(root, available) {
    all(root, 'button').forEach(node => disable(node, !available || node.hasAttribute('data-intelligence-unavailable')));
    all(root, 'summary').forEach(node => {
      attr(node, 'aria-disabled', !available); attr(node, 'tabindex', available ? 0 : -1);
    });
    all(root, '[data-intelligence-route]').forEach(node => {
      const enabled = available && destinationFor(node) !== null;
      attr(node, 'aria-disabled', !enabled); attr(node, 'tabindex', enabled ? 0 : -1);
      const reason = node.parentElement.querySelector('[data-intelligence-route-reason]');
      if (reason) hide(reason, enabled);
    });
    if (!available) all(root, 'details').forEach(node => { if (node.open) node.open = false; });
  }
  const disclosureID = node => (node.hasAttribute('data-intelligence-detail') ? 'detail:' : 'trace:')
    + (node.dataset.intelligenceDetail || node.dataset.intelligenceTrace);
  function apply() {
    if (!current) return;
    const {root, records, available} = current;
    for (const [id, {card}] of records) {
      hide(card, current.dismissed.has(id));
      all(card, 'details').forEach(node => { const open = available && !card.hidden && current.open.has(disclosureID(node)); if (node.open !== open) node.open = open; });
    }
    controls(root, available);
    text(root.querySelector('[data-intelligence-total]'), records.size);
    text(root.querySelector('[data-intelligence-remaining]'), records.size - current.dismissed.size);
    hide(root.querySelector('[data-intelligence-empty]'), current.dismissed.size !== records.size);
    hide(root.querySelector('[data-intelligence-source-reset]'), !current.reset);
    text(root.querySelector('[data-intelligence-availability]'), `${available ? 'Observed' : 'Retained'} anonymous sample: ${root.dataset.intelligenceLabel}. ${available ? 'Local inspection, dismissal and existing sample navigation only.' : 'Requested sample pending, failed or mismatched; local controls disabled.'}`);
  }
  function nativeGap(previous) {
    if (!previous || previous.root.isConnected || !host.isConnected
      || all(host, '#intelligence-command-console, [data-intelligence-root], .fetch-error').length) return false;
    const frames = all(host, '[data-island="render-envelope-dashboard"]'), pickers = all(host, '[data-sample-context-requested]');
    return frames.length === 0 && host.childElementCount === 0 || frames.length === 1
      && frames[0].getAttribute('aria-busy') === 'true' && frames[0].dataset.fragmentNavigation === 'island' && pickers.length === 1
      && pickers[0].dataset.sampleContextRequested === previous.context && pickers[0].dataset.sampleContextDisplayed === 'none';
  }
  function synchronize() {
    const root = owner(), next = validate(root), previous = current || handoff;
    if (!next) {
      all(host, '#intelligence-command-console, [data-intelligence-root]').forEach(node => {
        controls(node, false); text(node.querySelector('[data-intelligence-availability]'), 'Sample identity unavailable; local controls disabled. Live evaluation and Apply remain unavailable.');
      });
      handoff = nativeGap(previous) ? previous : null; current = null; report = null; return;
    }
    handoff = null;
    if (current?.root === root && current.key === next.key && current.available === next.available && current.mode === next.mode) {
      controls(root, current.available); settleReport(); return;
    }
    const same = previous?.key === next.key, replaced = previous && previous.root !== next.root;
    current = {...next, open:same ? previous.open : new Set(), dismissed:same ? previous.dismissed : new Set(),
      focus:same ? previous.focus : null, reset:same ? previous.reset : !!previous};
    if (!same || !next.available) report = null;
    apply();
    if (same && replaced) {
      const saved = current.focus; current.focus = null;
      const active = document.activeElement;
      if (saved && !saved.node.isConnected && saved.url === location.href && current.available && document.hasFocus()
        && (!active || active === document.body || !active.isConnected)) {
        const target = all(root, '[data-intelligence-focus]').find(node => node.dataset.intelligenceFocus === saved.token);
        if (visible(target) && !target.disabled && target.getAttribute('aria-disabled') !== 'true' && !target.closest('[hidden]')) target.focus({preventScroll:true});
      }
    }
    settleReport();
  }
"##;

const EVENTS: &str = r##"  function owned(event) {
    if (!(event.target instanceof Element) || !event.target.isConnected || !owner()?.contains(event.target)) return null;
    synchronize();
    const card = event.target.closest('[data-intelligence-card]');
    return current?.root.contains(event.target) && !event.target.closest('[hidden]')
      && (!card || !current.dismissed.has(card.dataset.intelligenceCard)) ? event.target : null;
  }
  document.addEventListener('click', event => {
    const target = owned(event);
    if (!target) {
      if (event.target instanceof Element && event.target.closest('#intelligence-command-console, [data-intelligence-root], [data-intelligence-card]')) event.preventDefault();
      return;
    }
    if (!current.available) { event.preventDefault(); return; }
    const close = target.closest('[data-intelligence-close]'), dismiss = target.closest('[data-intelligence-dismiss]');
    if (close) {
      const detail = close.closest('details'); current.open.delete(disclosureID(detail)); detail.open = false;
      if (document.hasFocus()) detail.querySelector(':scope > summary').focus();
    } else if (dismiss) {
      const id = dismiss.dataset.intelligenceDismiss;
      if (!current.records.has(id)) return;
      current.dismissed.add(id); current.open.delete('detail:' + id); current.open.delete('trace:' + id); apply();
      if (document.hasFocus()) current.root.querySelector('#intelligence-title').focus();
    } else if (target.closest('[data-intelligence-reset]')) {
      current.dismissed.clear(); current.open.clear(); current.reset = false; apply();
      if (document.hasFocus()) current.root.querySelector('[data-intelligence-reset]').focus();
    } else {
      const anchor = target.closest('[data-intelligence-route]'); if (!anchor) return;
      const destination = destinationFor(anchor), item = current.records.get(anchor.dataset.intelligenceRoute)?.item;
      if (!destination || item.channel && !openChannel(item.channel)) {
        event.preventDefault(); text(current.root.querySelector('[data-intelligence-route-status]'), 'Existing sample destination unavailable; no action output created.'); return;
      }
      if (current.mode === 'ssr') {
        const relation = nativeTab(destination);
        if (!relation) { event.preventDefault(); return; }
        if (relation.tab) {
          dispatching = true; try { relation.tab.click(); } finally { dispatching = false; }
          if (relation.tab.getAttribute('aria-selected') !== 'true' || !relation.panel.classList.contains('active')) { event.preventDefault(); return; }
        }
        if (!destination.hasAttribute('tabindex')) destination.tabIndex = -1;
        if (document.hasFocus()) destination.focus({preventScroll:true});
      }
      current.focus = null;
      text(current.root.querySelector('[data-intelligence-route-status]'), 'Opening existing sample; no action output created.');
      report = {key:current.key, destination:item.destination, channel:item.channel, url:anchor.href, expires:performance.now() + 5000};
      requestAnimationFrame(() => { synchronize(); settleReport(); });
      setTimeout(() => { synchronize(); settleReport(); }, 5000);
    }
  }, true);
  document.addEventListener('toggle', event => {
    const node = owned(event); if (!node?.matches('[data-intelligence-detail], [data-intelligence-trace]')) return;
    if (!current.available || node.closest('[hidden]')) { if (node.open) node.open = false; return; }
    if (node.open) current.open.add(disclosureID(node)); else current.open.delete(disclosureID(node));
  }, true);
  document.addEventListener('focusin', event => {
    if (dispatching) return;
    const node = owned(event)?.closest('[data-intelligence-focus]');
    if (current) current.focus = node && current.available ? {node, token:node.dataset.intelligenceFocus, url:location.href} : null;
    if (handoff && event.target !== document.body) handoff.focus = null;
  });
  const cancelFocus = () => { if (current) current.focus = null; if (handoff) handoff.focus = null; };
  for (const name of ['pointerdown','keydown']) document.addEventListener(name, event => {
    if (event.isTrusted && !dispatching) { report = null; if (event.target !== (current || handoff)?.focus?.node) cancelFocus(); }
  }, true);
  document.addEventListener('keydown', event => {
    if (!['Enter',' ','Spacebar'].includes(event.key) || !(event.target instanceof Element)) return;
    const target = event.target.closest('#intelligence-command-console, [data-intelligence-root], [data-intelligence-card]');
    if (target) { if (!owned(event) || !current?.available || event.target.getAttribute('aria-disabled') === 'true') event.preventDefault(); }
  }, true);
  window.addEventListener('blur', cancelFocus);
  for (const name of ['hashchange','popstate']) window.addEventListener(name, () => { cancelFocus(); if (report && location.href !== report.url) report = null; });
  new MutationObserver(() => {
    if (queued) return; queued = true;
    queueMicrotask(() => { queued = false; synchronize(); });
  }).observe(host, {childList:true, subtree:true, characterData:true, attributes:true,
    attributeFilter:['data-intelligence-root','data-intelligence-source','data-intelligence-context','data-intelligence-label','data-intelligence-hub-source',
      'data-intelligence-admitted','data-intelligence-available','data-intelligence-mode','data-intelligence-card','data-intelligence-record','data-intelligence-field',
      'data-intelligence-route','data-intelligence-channel','data-intelligence-detail','data-intelligence-trace','data-intelligence-dismiss','data-intelligence-close','data-intelligence-focus',
      'data-intelligence-route-reason','data-sample-context-requested','data-sample-context-displayed','aria-busy','data-fragment-navigation',
      'id','href','role','aria-controls','aria-labelledby','aria-selected','aria-describedby','disabled','hidden','class','style','data-hub-source','data-hub-context','data-hub-mode',
      'data-workflow-source','data-workflow-context','data-workflow-mode','data-identity-panel','data-finance-panel','data-comms-board-surface']});
  synchronize();
})();
"##;

#[cfg(test)]
#[path = "intelligence_sample_test.rs"]
mod tests;
