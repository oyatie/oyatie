use std::process::Command;

use super::SCRIPT;

const HANDOFF_CHECK: &str = r#"
const assert = require('node:assert/strict'), vm = require('node:vm');
function fixture() {
  const documentEvents = {}, windowEvents = {}, frames = [], microtasks = [], mutations = [];
  let observer, options, latest;
  const emit = (events, name, event) => (events[name] || []).forEach(callback => callback(event));
  const attr = key => 'data-' + key.replace(/[A-Z]/g, letter => '-' + letter.toLowerCase());
  function changed(target, attributeName, type = 'attributes') {
    if (observer && host.contains(target) && (type !== 'attributes' || options.attributeFilter.includes(attributeName))) mutations.push({target, attributeName, type});
  }
  class Element {
    constructor(id = '', data = {}, classes = '', tag = 'div') {
      this.id = id; this.tag = tag; this.children = []; this.attrs = {}; this.isConnected = true; this.value = ''; this._text = ''; this.clicks = 0; this.focusCount = 0;
      this.dataset = new Proxy({...data}, {set: (object, key, value) => { object[key] = value; changed(this, attr(key)); return true; }});
      const names = new Set(classes.split(' ').filter(Boolean));
      this.classList = {contains: name => names.has(name), toggle: (name, active) => active ? names.add(name) : names.delete(name)};
    }
    get disabled() { return this.hasAttribute('disabled'); }
    set disabled(value) { if (value) this.attrs.disabled = ''; else delete this.attrs.disabled; changed(this, 'disabled'); }
    get textContent() { return this._text; }
    set textContent(value) { this._text = value; changed(this, null, 'characterData'); }
    get title() { return this.getAttribute('title') || ''; }
    set title(value) { this.setAttribute('title', value); }
    get href() { return new URL(this.getAttribute('href'), location.href).href; }
    append(child) { child.parentElement = this; this.children.push(child); changed(this, null, 'childList'); return child; }
    contains(target) { for (let node = target; node; node = node.parentElement) if (node === this) return true; return false; }
    getAttribute(name) {
      if (name === 'id') return this.id;
      if (name.startsWith('data-')) return this.dataset[name.slice(5).replace(/-([a-z])/g, (_, letter) => letter.toUpperCase())] ?? null;
      return this.attrs[name] ?? null;
    }
    hasAttribute(name) { return this.getAttribute(name) !== null; }
    setAttribute(name, value) { this.attrs[name] = String(value); changed(this, name); }
    removeAttribute(name) { delete this.attrs[name]; changed(this, name); }
    matches(selector) {
      return selector.split(',').some(part => {
        part = part.trim();
        const attributes = [...part.matchAll(/\[([^=\]]+)(?:="([^"]*)")?\]/g)];
        const plain = part.replace(/\[[^\]]+\]/g, ''), tag = plain.match(/^[a-z]+/)?.[0];
        return (!tag || this.tag === tag) && [...plain.matchAll(/#([\w-]+)/g)].every(match => this.id === match[1])
          && [...plain.matchAll(/\.([\w-]+)/g)].every(match => this.classList.contains(match[1]))
          && attributes.every(([, name, value]) => value === undefined ? this.hasAttribute(name) : this.getAttribute(name) === value);
      });
    }
    closest(selector) { for (let node = this; node; node = node.parentElement) if (node.matches(selector)) return node; return null; }
    querySelectorAll(selector) {
      const descendants = this.children.flatMap(child => [child, ...child.querySelectorAll('*')]);
      return descendants.filter(node => selector.split(',').some(part => {
        const pieces = part.trim().split(/\s+/); if (!node.matches(pieces.pop())) return false;
        let ancestor = node.parentElement;
        for (const piece of pieces.reverse()) { while (ancestor && !ancestor.matches(piece)) ancestor = ancestor.parentElement; if (!ancestor) return false; ancestor = ancestor.parentElement; }
        return true;
      }));
    }
    querySelector(selector) { return this.querySelectorAll(selector)[0] || null; }
    getClientRects() { return this.isConnected ? [{}] : []; }
    getBoundingClientRect() { return {width: 10, height: 10, top: 1, bottom: 11, left: 1, right: 11}; }
    focus() { this.focusCount++; document.activeElement = this; emit(documentEvents, 'focusin', {target: this}); }
    scrollIntoView() {}
    click() { this.clicks++; this.onClick?.(); }
  }
  const body = new Element('body'), host = body.append(new Element('dashboard-island-root'));
  const find = (node, id) => node.id === id ? node : node.children.map(child => find(child, id)).find(Boolean);
  const document = {body, activeElement: body, hasFocus: () => true, getElementById: id => find(body, id), addEventListener: (name, callback) => (documentEvents[name] ||= []).push(callback)};
  const window = {addEventListener: (name, callback) => (windowEvents[name] ||= []).push(callback)};
  const url = new URL('https://preview.invalid/#evidence-spine'), location = {get href() { return url.href; }};
  const fields = {event: ['State', 'Receipt', 'Title', 'Body', 'Source', 'Owner', 'SLA'], node: ['Key', 'Label', 'Value', 'Description'], audit: ['Time', 'Event', 'Actor', 'Receipt', 'State'], source: ['Key', 'Title', 'Receipt', 'Route', 'Owner', 'State']};
  const records = Object.entries({event: 6, node: 8, audit: 5, source: 4}).flatMap(([family, count]) => Array.from({length: count}, (_, index) => {
    const pairs = fields[family].map(name => [name, name === 'State' ? 'sealed' : `${family}-${index}-${name} sample`]);
    const values = new Map(pairs), suffix = values.get(family === 'node' || family === 'source' ? 'Key' : 'Receipt');
    return [`${family}:${suffix}`, pairs];
  }));
  const routes = Array.from({length: 40}, (_, index) => [`route-${index}`, index ? 'evidence-spine' : 'work-hub', index ? null : 'Mail']);
  function attach(mode, enabled = true) {
    const source = JSON.stringify(['tenant-admin', 'Tenant sample', 'native-hub-source', 'Sample workflow', 'Sample goal', records, routes]);
    const data = {aeSource: source, aeContext: 'tenant-admin', aeLabel: 'Tenant sample', aeHubSource: 'native-hub-source', aeMode: mode, aeSourceAvailable: 'true', aeAvailable: 'true'};
    const evidence = host.append(new Element('evidence-spine', data)), audit = host.append(new Element('resource-audit-console', data));
    const add = (parent, dataset, text = '', tag = 'div') => { const node = parent.append(new Element('', dataset, '', tag)); node.textContent = text; return node; };
    for (const owner of [evidence, audit]) {
      const inspection = add(owner, {aeInspection: ''}); add(inspection, {aeInspectTitle: ''});
      for (let index = 0; index < 7; index++) { const slot = add(inspection, {aeInspectField: String(index)}); add(slot, {aeInspectLabel: ''}); add(slot, {aeInspectValue: ''}); }
      for (const key of ['aeClose', 'aeAvailability', 'aeReset', 'aeRouteStatus', 'aeUnavailableReason', 'aeFixture']) add(owner, {[key]: ''});
    }
    const panel = audit.append(new Element('resource-panel-audit'));
    records.forEach(([token, pairs]) => {
      const family = token.split(':')[0], values = new Map(pairs), parent = ['event', 'node'].includes(family) ? evidence : panel;
      const row = add(parent, family === 'event' ? {evidenceEvent: '', evidenceState: values.get('State')} : {aeRecordRow: ''});
      const extra = family === 'node' ? {objectNode: values.get('Key'), objectLabel: values.get('Label')} : family === 'source'
        ? Object.fromEntries(['receiptSource', 'receiptTitle', 'receiptId', 'receiptRoute', 'receiptOwner', 'receiptState'].map((key, index) => [key, pairs[index][1]])) : {};
      const control = add(row, {aeSelect: token, aeRecord: JSON.stringify(pairs), aeFocus: token, ...extra}, '', 'button');
      pairs.forEach(([name, value]) => add(family === 'event' ? control : row, {aeValue: name}, value));
    });
    routes.forEach(([key, route, channel]) => { const link = add(evidence, {aeRouteKey: key, aeRoute: route, aeFocus: key, ...(channel ? {aeChannel: channel} : {})}, '', 'a'); link.setAttribute('href', '#' + route); });
    for (const effect of ['evidence:run-review', 'evidence:export', 'evidence:attach', 'trust:seal-receipt', 'trust:trace-lineage', 'audit:seal-packet', 'stitch:seal']) { const node = add(evidence, {aeUnavailable: 'true', aeEffect: effect}, 'Unavailable'); node.title = 'No live effect'; }
    for (const [kind, text] of [['name', 'Sample workflow'], ['name', 'Sample workflow'], ['goal', 'Sample goal']]) add(evidence, {aeWorkflow: kind}, text);
    for (const key of ['evidenceSearch', 'aeClear', 'aeEmpty', 'evidenceStatus']) add(evidence, {[key]: ''});
    for (const filter of ['all', 'blocking', 'review', 'sealed', 'watch']) add(evidence, {evidenceFilter: filter}, '', 'button');
    const resourceTabs = audit.append(new Element('', {}, 'resource-tabs')), resourceTab = resourceTabs.append(new Element('resource-tab-audit', {resourceTab: 'audit'}));
    resourceTab.setAttribute('role', 'tab'); resourceTab.setAttribute('aria-selected', 'true');
    for (let index = 0; index < 7; index++) add(audit, {[index ? 'resourceAction' : 'resourceSearch']: ''});
    add(audit, {resourceStatus: ''}, 'Native toolbar');
    const hub = host.append(new Element('work-hub')), workspace = hub.append(new Element('', {hubContext: 'tenant-admin', hubSource: 'native-hub-source', hubMode: mode}, 'communications-hub'));
    const board = add(workspace, {commsProductBoard: '', commsBoardSurface: mode === 'ssr' ? 'Mail' : 'Messenger'});
    workspace.append(new Element('', {}, 'hub-list')); workspace.append(new Element('', {}, 'hub-detail'));
    const composer = workspace.append(new Element('', {}, 'hub-composer')).append(new Element('', {}, '', 'textarea')); composer.disabled = !enabled;
    const tabs = Object.fromEntries(['Mail', 'Community', 'Messenger'].map(channel => { const tab = workspace.append(new Element('hub-tab-' + channel.toLowerCase())); tab.setAttribute('aria-selected', String(board.dataset.commsBoardSurface === channel)); tab.disabled = mode === 'ssr' || !enabled; return [channel, tab]; }));
    latest = {evidence, audit, hub, workspace, composer, tabs, board}; return latest;
  }
  attach('ssr');
  vm.runInNewContext(process.argv[1], {document, window, location, Element, innerHeight: 100, innerWidth: 100,
    requestAnimationFrame: callback => frames.push(callback), queueMicrotask: callback => microtasks.push(callback),
    MutationObserver: class {constructor(callback) { observer = callback; } observe(_, config) { options = config; }}}, {timeout: 1000});
  function drain() {
    for (let count = 0; mutations.length || microtasks.length || frames.length; count++) {
      assert.ok(count < 30, 'observer, microtask and frame queues settle without loops');
      if (mutations.length) observer(mutations.splice(0));
      while (microtasks.length) microtasks.shift()();
      if (frames.length) frames.shift()();
    }
  }
  function click(target) {
    const event = {target, button: 0, preventDefault() {this.defaultPrevented = true;}}; emit(documentEvents, 'click', event);
    drain();
    if (!event.defaultPrevented && target.tag === 'a') { const oldURL = url.href; url.hash = target.getAttribute('href'); emit(windowEvents, 'popstate', {}); emit(windowEvents, 'hashchange', {oldURL, newURL: url.href}); }
    drain();
  }
  function detach() {
    function disconnect(node) { node.isConnected = false; node.children.forEach(disconnect); }
    for (const child of host.children) { disconnect(child); child.parentElement = null; } host.children = [];
    document.activeElement = body; changed(host, null, 'childList'); drain();
  }
  function reattach(saved) {
    function connect(node) { node.isConnected = true; node.children.forEach(connect); }
    for (const node of [saved.evidence, saved.audit, saved.hub]) { connect(node); host.append(node); } latest = saved; drain();
  }
  drain(); assert.equal(latest.evidence.querySelector('[data-evidence-search]').disabled, false, 'complete native fixture passes production validation');
  return {get current() {return latest;}, body, document, url, windowEvents, documentEvents, emit, attach, detach, reattach, drain, click,
    route: () => click(latest.evidence.querySelector('[data-ae-route-key="route-0"]')),
    select: channel => { latest.board.dataset.commsBoardSurface = channel; for (const [name, tab] of Object.entries(latest.tabs)) tab.setAttribute('aria-selected', String(name === channel)); drain(); }};
}
const cases = {
  local() {
    const state = fixture(), old = state.current, search = old.evidence.querySelector('[data-evidence-search]');
    search.value = 'sample'; state.emit(state.documentEvents, 'input', {target: search});
    state.click(old.evidence.querySelector('[data-evidence-filter="sealed"]')); state.click(old.evidence.querySelector('[data-ae-select]'));
    state.detach(); assert.equal(search.disabled, true, 'absent owner disables retained controls');
    assert.equal(old.evidence.querySelector('[data-ae-inspection]').hidden, true, 'absence exposes no inspection');
    const fresh = state.attach('island'); state.drain();
    assert.equal(fresh.evidence.querySelector('[data-evidence-search]').value, 'sample', 'matching verified owner restores local query across physical absence');
    assert.equal(fresh.evidence.querySelector('[data-evidence-filter="sealed"]').getAttribute('aria-pressed'), 'true', 'filter survives absence');
    assert.equal(fresh.evidence.querySelector('[data-ae-select]').getAttribute('aria-pressed'), 'true', 'selection survives absence');
    assert.equal(fresh.evidence.querySelector('[data-ae-reset]').hidden, true, 'matching source has no reset warning');
  },
  handoff() {
    const state = fixture(); state.route(); state.detach(); const fresh = state.attach('island', false); state.drain();
    assert.equal(fresh.tabs.Mail.clicks, 0, 'disabled native Hub is never triggered');
    fresh.tabs.Mail.disabled = false; state.drain(); assert.equal(fresh.tabs.Mail.clicks, 0, 'disabled composer also gates native handoff');
    fresh.composer.disabled = false; state.drain();
    assert.equal(fresh.tabs.Mail.clicks, 1, 'AE-identical enabled Hub resumes retained Mail intent');
    state.drain(); assert.equal(fresh.tabs.Mail.clicks, 1, 'unconfirmed native click is triggered once');
    state.select('Mail'); state.select('Messenger');
    assert.equal(fresh.tabs.Mail.clicks, 1, 'confirmed Mail consumes intent and cannot reopen after native choice');
    assert.equal(state.document.activeElement, state.body, 'channel handoff does not steal focus');
  },
  navigation() {
    const state = fixture(), oldURL = state.url.href, route = state.current.evidence.querySelector('[data-ae-route-key="route-0"]');
    state.emit(state.documentEvents, 'click', {target: route, button: 0, preventDefault() {this.defaultPrevented = true;}}); state.drain();
    state.detach(); const fresh = state.attach('island'); state.drain();
    assert.equal(state.url.href, oldURL, 'native observer and matching island can settle before anchor default navigation');
    assert.equal(fresh.tabs.Mail.disabled || fresh.composer.disabled, false, 'pre-navigation native Hub is ready');
    assert.equal(fresh.tabs.Mail.clicks, 0, 'matching ready Hub waits at the exact initiating URL before its own navigation');
    state.url.hash = '#work-hub'; state.emit(state.windowEvents, 'popstate', {}); state.emit(state.windowEvents, 'hashchange', {oldURL, newURL: state.url.href});
    state.drain();
    assert.equal(fresh.tabs.Mail.clicks, 1, 'committed own navigation resumes one matching native Mail attempt');
    const other = fixture(), link = other.current.evidence.querySelector('[data-ae-route-key="route-0"]');
    other.emit(other.documentEvents, 'click', {target: link, button: 0, preventDefault() {this.defaultPrevented = true;}}); other.drain();
    other.url.hash = '#other-destination'; other.current.evidence.querySelector('[data-ae-route-status]').textContent = 'Other navigation'; other.drain();
    other.detach(); const returned = other.attach('island'); other.drain();
    assert.equal(returned.tabs.Mail.clicks, 0, 'a URL other than the initiating or requested URL cancels pending navigation');
    for (const kind of ['foreign', 'malformed', 'disabled', 'unavailable']) {
      const guarded = fixture(), oldURL = guarded.url.href, link = guarded.current.evidence.querySelector('[data-ae-route-key="route-0"]');
      guarded.emit(guarded.documentEvents, 'click', {target: link, button: 0, preventDefault() {this.defaultPrevented = true;}}); guarded.drain();
      guarded.detach(); const matching = guarded.attach('island'); guarded.drain();
      for (const owner of [matching.evidence, matching.audit]) { if (kind === 'foreign') owner.dataset.aeHubSource = 'foreign'; else if (kind === 'malformed') owner.dataset.aeSource = '{'; else owner.dataset[kind === 'disabled' ? 'aeSourceAvailable' : 'aeAvailable'] = 'false'; }
      guarded.url.hash = '#work-hub'; guarded.emit(guarded.windowEvents, 'popstate', {}); guarded.emit(guarded.windowEvents, 'hashchange', {oldURL, newURL: guarded.url.href}); guarded.drain();
      assert.equal(matching.tabs.Mail.clicks, 0, `${kind} current source refuses own-navigation replay before observer delivery`);
      assert.equal(matching.evidence.querySelector('[data-evidence-search]').disabled, true, `${kind} current source keeps controls unavailable`);
      guarded.detach(); const restored = guarded.attach('island'); guarded.drain();
      assert.equal(restored.tabs.Mail.clicks, 0, `${kind} refusal consumes intent permanently`);
    }
    const gap = fixture(), gapURL = gap.url.href, gapLink = gap.current.evidence.querySelector('[data-ae-route-key="route-0"]'), retained = gap.current;
    gap.emit(gap.documentEvents, 'click', {target: gapLink, button: 0, preventDefault() {this.defaultPrevented = true;}}); gap.drain(); gap.detach();
    gap.url.hash = '#work-hub'; gap.emit(gap.windowEvents, 'popstate', {}); gap.emit(gap.windowEvents, 'hashchange', {oldURL: gapURL, newURL: gap.url.href}); gap.drain();
    assert.equal(retained.evidence.querySelector('[data-evidence-search]').disabled, true, 'own navigation during owner gap keeps retained controls unavailable');
    const resumed = gap.attach('island'); gap.drain();
    assert.equal(resumed.tabs.Mail.clicks, 1, 'matching available owner return resumes navigation committed during absence');
  },
  refusals() {
    for (const kind of ['foreign', 'malformed', 'disabled', 'unavailable']) {
      const state = fixture(); state.route(); state.detach(); const fresh = state.attach('island', false);
      for (const owner of [fresh.evidence, fresh.audit]) { if (kind === 'foreign') owner.dataset.aeHubSource = 'foreign'; else if (kind === 'malformed') owner.dataset.aeSource = '{'; else owner.dataset[kind === 'disabled' ? 'aeSourceAvailable' : 'aeAvailable'] = 'false'; }
      state.drain(); assert.equal(fresh.evidence.querySelector('[data-evidence-search]').disabled, true, `${kind} source fails closed`);
      state.detach(); const matching = state.attach('island'); state.drain(); assert.equal(matching.tabs.Mail.clicks, 0, `${kind} source cancels pending permanently`);
    }
    const mismatch = fixture(); mismatch.route(); mismatch.detach(); const fresh = mismatch.attach('island'); fresh.workspace.dataset.hubSource = 'foreign'; mismatch.drain();
    assert.equal(fresh.tabs.Mail.clicks, 0, 'foreign native Hub cannot receive retained intent');
    const foreign = fixture(), query = foreign.current.evidence.querySelector('[data-evidence-search]'); query.value = 'sample'; foreign.emit(foreign.documentEvents, 'input', {target: query});
    foreign.route(); foreign.detach(); const other = foreign.attach('island');
    for (const owner of [other.evidence, other.audit]) { const source = JSON.parse(owner.dataset.aeSource); source[2] = 'other-native-source'; owner.dataset.aeSource = JSON.stringify(source); owner.dataset.aeHubSource = source[2]; }
    other.workspace.dataset.hubSource = 'other-native-source'; foreign.drain();
    assert.equal(other.evidence.querySelector('[data-evidence-search]').disabled, false, 'different complete source passes fresh native validation');
    assert.equal(other.evidence.querySelector('[data-evidence-search]').value, '', 'fresh foreign source cannot import saved query');
    assert.equal(other.tabs.Mail.clicks, 0, 'fresh foreign source cannot import old Mail intent');
    assert.equal(other.evidence.querySelector('[data-ae-reset]').hidden, false, 'different verified source reports reset');
    const reused = fixture(); reused.route(); const saved = reused.current; reused.detach();
    for (const owner of [saved.evidence, saved.audit]) owner.dataset.aeSourceAvailable = 'false';
    reused.reattach(saved); reused.detach(); const enabled = reused.attach('island'); reused.drain();
    assert.equal(enabled.tabs.Mail.clicks, 0, 'same-element unavailable source cancels intent even after an absent-owner gap');
  },
  cancellation() {
    for (const name of ['pointerdown', 'keydown', 'blur', 'popstate', 'hashchange']) {
      const state = fixture(); state.route(); state.detach();
      state.emit(['pointerdown', 'keydown'].includes(name) ? state.documentEvents : state.windowEvents, name, {isTrusted: true, target: state.body});
      const fresh = state.attach('island'); state.drain(); assert.equal(fresh.tabs.Mail.clicks, 0, `later ${name} cancels retained handoff`);
    }
  }
};
for (const [name, run] of Object.entries(cases)) if (!process.argv[2] || process.argv[2] === name) { run(); console.log(`Audit Evidence ${name} regression PASS`); }
"#;

#[test]
fn audit_evidence_retains_matching_source_and_confirms_native_handoff() {
    let output = Command::new("node")
        .args(["-e", HANDOFF_CHECK, SCRIPT])
        .output()
        .expect("Node runs the native Audit and Evidence bridge regression");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
