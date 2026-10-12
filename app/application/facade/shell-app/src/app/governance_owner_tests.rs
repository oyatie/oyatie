use std::{
    io::Write,
    process::{Command, Stdio},
};

use super::*;

pub(crate) fn check(script: &str) {
    let fixtures = OperatorContext::ALL.map(|context| {
        let envelope = server_derived_envelope(context);
        serde_json::json!({
            "context": context.id(),
            "governance": static_governance_preview(&envelope),
            "hub": communication_hub_html(&envelope, HubChannel::Messenger),
            "rails": governance_rail_html(context, false) + &governance_rail_html(context, true),
        })
    });
    let mut child = Command::new("node")
        .args(["-e", OWNER_CHECK, script])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("node must be available for the Governance owner regression");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&fixtures).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    print!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

const OWNER_CHECK: &str = r##"
const assert = require('node:assert/strict'), vm = require('node:vm');
const fixtures = JSON.parse(require('node:fs').readFileSync(0, 'utf8'));
const decode = text => text.replace(/&(amp|lt|gt|quot|apos|#39|#\d+|#x[\da-f]+);/gi, (_, entity) =>
  ({amp:'&',lt:'<',gt:'>',quot:'"',apos:"'",'#39':"'"})[entity] ?? String.fromCodePoint(parseInt(entity.slice(entity[1] === 'x' ? 2 : 1), entity[1] === 'x' ? 16 : 10)));
function fixture(native = fixtures[0], initial = true) {
  const microtasks = [], listeners = {}, windowListeners = {}, observers = [];
  let document, deliveries = 0, genericActivations = 0;
  const mutation = (target, type, attributeName) => {
    for (const observer of observers) {
      if (!observer.targets.some(({node, options}) => (node === target || options.subtree && node.contains(target))
        && options[type] && (type !== 'attributes' || !options.attributeFilter || options.attributeFilter.includes(attributeName)))) continue;
      if (!observer.queued) { observer.queued = true; microtasks.push(() => { observer.queued = false; deliveries++; observer.callback(); }); }
    }
  };
  class Element {
    constructor(tag = 'div') { this.tagName = tag.toUpperCase(); this.attrs = {}; this.childNodes = []; this.parentElement = null; this.focusCount = 0; this.clickCount = 0; this.scrollLeft = this.scrollTop = 0; this.value = ''; }
    get children() { return this.childNodes.filter(node => node instanceof Element); }
    get firstElementChild() { return this.children[0] ?? null; }
    get isConnected() { return this === document?.body || !!this.parentElement?.isConnected; }
    get id() { return this.getAttribute('id') ?? ''; } set id(value) { this.setAttribute('id', value); }
    get dataset() { return new Proxy({}, {get: (_, key) => this.getAttribute('data-' + key.replace(/[A-Z]/g, c => '-' + c.toLowerCase())) ?? undefined,
      set: (_, key, value) => { this.setAttribute('data-' + key.replace(/[A-Z]/g, c => '-' + c.toLowerCase()), value); return true; }}); }
    get classList() { const values = () => (this.getAttribute('class') ?? '').split(/\s+/).filter(Boolean); return {
      contains: value => values().includes(value), toggle: (value, enabled) => { const set = new Set(values()); enabled ? set.add(value) : set.delete(value); this.setAttribute('class', [...set].join(' ')); }}; }
    get textContent() { return this.childNodes.map(node => typeof node === 'string' ? node : node.textContent).join(''); }
    set textContent(value) { for (const node of this.children) node.parentElement = null; this.childNodes = [String(value)]; mutation(this, 'childList'); }
    get disabled() { return this.hasAttribute('disabled'); } set disabled(value) { value ? this.setAttribute('disabled', '') : this.removeAttribute('disabled'); }
    get hidden() { return this.hasAttribute('hidden'); } set hidden(value) { value ? this.setAttribute('hidden', '') : this.removeAttribute('hidden'); }
    get open() { return this.hasAttribute('open'); } set open(value) { value ? this.setAttribute('open', '') : this.removeAttribute('open'); }
    get href() { return new URL(this.getAttribute('href') ?? '', location.href).href; }
    setAttribute(name, value) { this.attrs[name] = String(value); mutation(this, 'attributes', name); }
    getAttribute(name) { return this.attrs[name] ?? null; }
    hasAttribute(name) { return name in this.attrs; }
    removeAttribute(name) { delete this.attrs[name]; mutation(this, 'attributes', name); }
    append(node) { if (node instanceof Element) { node.remove(); node.parentElement = this; } this.childNodes.push(node); mutation(this, 'childList'); return node; }
    remove() { if (!this.parentElement) return; const parent = this.parentElement; parent.childNodes = parent.childNodes.filter(node => node !== this); this.parentElement = null; if (this.contains(document.activeElement)) document.activeElement = document.body; mutation(parent, 'childList'); }
    replaceWith(node) { const parent = this.parentElement, index = parent.childNodes.indexOf(this); this.remove(); node.remove(); node.parentElement = parent; parent.childNodes.splice(index, 0, node); mutation(parent, 'childList'); }
    contains(target) { for (let node = target; node; node = node.parentElement) if (node === this) return true; return false; }
    matches(selector) { return selector.split(',').some(part => {
      const chain = part.trim().split(/\s+(?![^[]*\])/), compound = chain.pop();
      const tag = compound.match(/^[\w-]+/); if (tag && this.tagName !== tag[0].toUpperCase()) return false;
      for (const [, id] of compound.matchAll(/#([\w-]+)/g)) if (this.id !== id) return false;
      for (const [, cls] of compound.matchAll(/\.([\w-]+)/g)) if (!this.classList.contains(cls)) return false;
      for (const [, name, value] of compound.matchAll(/\[([\w-]+)(?:=["']?([^\]"']+)["']?)?\]/g)) if (!this.hasAttribute(name) || value !== undefined && this.getAttribute(name) !== value) return false;
      let ancestor = this.parentElement;
      while (chain.length) { const next = chain.pop(); while (ancestor && !ancestor.matches(next)) ancestor = ancestor.parentElement; if (!ancestor) return false; ancestor = ancestor.parentElement; }
      return true;
    }); }
    closest(selector) { for (let node = this; node; node = node.parentElement) if (node.matches(selector)) return node; return null; }
    querySelectorAll(selector) { const found = []; const visit = node => { for (const child of node.children) { if (child.matches(selector)) found.push(child); visit(child); } }; visit(this); return found; }
    querySelector(selector) { return this.querySelectorAll(selector)[0] ?? null; }
    set innerHTML(html) { for (const child of this.children) child.parentElement = null; this.childNodes = []; parse(html, this); mutation(this, 'childList'); }
    cloneNode(deep) { const clone = new Element(this.tagName); clone.attrs = {...this.attrs}; if (this.content) clone.content = this.content.cloneNode(deep); if (deep) for (const child of this.childNodes) clone.append(child instanceof Element ? child.cloneNode(true) : child); return clone; }
    getClientRects() { return this.isConnected && !this.hidden ? [{}] : []; }
    focus() { if (!this.isConnected || this.disabled) return; this.focusCount++; document.activeElement = this; emit('focusin', this); }
    scrollIntoView() {}
    click() { this.clickCount++; return emit('click', this); }
  }
  function parse(html, root) {
    const stack = [root];
    for (const token of html.match(/<!--[\s\S]*?-->|<\/?[\w-]+(?:\s[^<>]*?)?\/?>|[^<]+/g) ?? []) {
      if (token.startsWith('<!--')) continue;
      if (token.startsWith('</')) { if (stack.length > 1) stack.pop(); continue; }
      if (!token.startsWith('<')) { stack.at(-1).append(decode(token)); continue; }
      const tag = token.match(/^<([\w-]+)/)[1], node = new Element(tag);
      for (const [, name, quoted, unquoted] of token.slice(tag.length + 1, -1).matchAll(/([\w:-]+)(?:\s*=\s*(?:"([^"]*)"|([^\s>]+)))?/g)) node.attrs[name] = decode(quoted ?? unquoted ?? '');
      stack.at(-1).append(node);
      if (tag === 'template') { node.content = new Element('fragment'); stack.push(node.content); }
      else if (!['input','img','br','hr','meta','link','source','wbr'].includes(tag) && !token.endsWith('/>')) stack.push(node);
    }
    return root;
  }
  const body = new Element('body');
  document = { body, activeElement: body, hasFocus: () => true, createElement: tag => new Element(tag),
    getElementById: id => body.querySelector('#' + id), querySelectorAll: selector => body.querySelectorAll(selector),
    addEventListener: (name, callback, capture = false) => (listeners[name] ??= []).push({callback, capture}) };
  const host = body.append(new Element()); host.id = 'dashboard-island-root';
  const hubOwner = host.append(new Element()); hubOwner.id = 'work-hub'; hubOwner.innerHTML = native.hub;
  const railHost = body.append(new Element()); railHost.innerHTML = native.rails + native.rails;
  const rails = () => document.querySelectorAll('[data-governance-rail]');
  const unrelated = body.append(new Element('a')); unrelated.setAttribute('href', '#payroll-cockpit');
  const payroll = host.append(new Element()); payroll.id = 'payroll-cockpit';
  const location = {href: 'https://preview.invalid/'};
  const window = {addEventListener: (name, callback) => (windowListeners[name] ??= []).push(callback)};
  function emit(name, target, extra = {}) {
    const event = {type: name, target, button: 0, isTrusted: false, defaultPrevented: false, preventDefault() { this.defaultPrevented = true; }, ...extra};
    event.documentDelivered = target.isConnected && document.body.contains(target);
    if (!event.documentDelivered) return event;
    for (const {callback} of (listeners[name] ?? []).filter(row => row.capture)) callback(event);
    for (const {callback} of (listeners[name] ?? []).filter(row => !row.capture)) callback(event);
    return event;
  }
  function attach(html = native.governance) { const container = new Element(); container.innerHTML = html; const root = container.firstElementChild; host.append(root); return root; }
  let root = initial ? attach() : null;
  vm.runInNewContext(process.argv[1], {document, window, location, Element, requestAnimationFrame: callback => microtasks.push(callback), queueMicrotask: callback => microtasks.push(callback),
    MutationObserver: class { constructor(callback) { this.callback = callback; this.targets = []; observers.push(this); } observe(node, options) { this.targets.push({node, options}); } }}, {timeout: 1000});
  document.addEventListener('click', event => { if (event.defaultPrevented) return; const anchor = event.target.closest('a'); if (anchor) { genericActivations++; location.href = anchor.href; } });
  function flush() { for (let count = 0; microtasks.length; count++) { assert.ok(count < 30, 'idempotent projection must stop mutation feedback'); microtasks.shift()(); } }
  flush();
  function unavailable(nodes = [root], hiddenRails = true) {
    for (const node of nodes.filter(Boolean)) {
      assert.ok(node.querySelectorAll('button, input').every(control => control.disabled), 'unavailable owner disables retained controls');
      assert.ok(node.querySelectorAll('summary, [data-governance-route]').every(control => control.getAttribute('aria-disabled') === 'true'), 'unavailable owner marks native summary/routes');
    }
    if (hiddenRails) assert.ok(rails().every(rail => rail.hidden), 'unavailable owner hides all external Governance rails');
  }
  function cancelled(control) {
    assert.ok(control.isConnected && document.body.contains(control), 'document cancellation requires a connected event path');
    for (const [name, extra] of [['click', {}], ['keydown', {key: 'Enter'}], ['keydown', {key: ' '}], ['keydown', {key: 'Spacebar'}]]) {
      const before = genericActivations, url = location.href;
      const event = emit(name, control, extra);
      assert.ok(event.documentDelivered && event.defaultPrevented, `${name}/${extra.key ?? ''} must cancel unavailable Governance defaults`);
      assert.equal(genericActivations, before, 'generic fragment consumer respects capture cancellation'); assert.equal(location.href, url);
    }
  }
  function available(node = root) {
    assert.ok(rails().every(rail => !rail.hidden));
    assert.ok(node.querySelectorAll('button, input').every(control => control.disabled === (control.dataset.governanceUnavailable === 'true')));
    assert.ok(node.querySelectorAll('summary, [data-governance-route]').every(control => control.getAttribute('aria-disabled') === 'false'));
  }
  return {document, host, body, native, location, hubOwner, attach, flush, emit, unavailable, available, cancelled, rails, unrelated,
    get root() { return root; }, set root(value) { root = value; }, get deliveries() { return deliveries; }, get genericActivations() { return genericActivations; }};
}
function seed(state) {
  const root = state.root, search = root.querySelector('[data-logic-search]');
  search.value = 'payroll'; state.emit('input', search); search.focus();
  const attention = root.querySelector('[data-logic-filter="attention"]'); state.emit('click', attention);
  assert.equal(search.value, 'payroll'); assert.equal(attention.getAttribute('aria-pressed'), 'true');
  const mail = root.querySelector('[data-governance-channel="Mail"]');
  assert.ok(mail && !state.emit('click', mail).defaultPrevented, 'valid source and Hub binding admit existing SSR sample');
  state.flush();
  search.focus();
  return {search, attention, url: state.location.href, route: root.querySelector('[data-governance-route]'), summary: root.querySelector('summary')};
}
const identityProof = [];
for (const native of fixtures.filter(row => row.context !== 'healthcare-clinician')) {
  for (const identity of ['removed', 'renamed']) for (const timing of ['alone', 'sequential', 'atomic']) {
    const state = fixture(native), old = state.root, saved = seed(state);
    let replacement;
    const probe = (claim, action) => { try { action(); identityProof.push({context:native.context, identity, timing, claim, result:'PASS'}); }
      catch (error) { identityProof.push({context:native.context, identity, timing, claim, result:'FAIL', error:error.message}); } };
    if (identity === 'removed') old.removeAttribute('id'); else old.id = 'retained-governance-preview';
    if (timing !== 'atomic') state.flush();
    if (timing !== 'alone') {
      replacement = state.attach(); replacement.dataset.governanceMode = 'island'; state.root = replacement;
      state.hubOwner.innerHTML = native.hub;
      const hub = state.hubOwner.querySelector('[data-hub-source]'); hub.dataset.hubMode = 'island';
      hub.querySelectorAll('button, textarea').forEach(control => { control.disabled = false; });
      const tab = hub.querySelector('#hub-tab-mail'), clicks = tab.clickCount;
      state.flush();
      probe('new owner admitted independently', () => { state.available(replacement); assert.equal(replacement.querySelector('[data-logic-search]').value, ''); });
      probe('old pending channel not replayed', () => assert.equal(tab.clickCount, clicks));
    }
    probe('former connected scope projected unavailable', () => state.unavailable([old], !replacement));
    probe('former connected route default cancelled', () => state.cancelled(saved.route));
    probe('former connected summary default cancelled', () => state.cancelled(saved.summary));
    if (replacement) {
      state.location.href = saved.url;
      old.remove(); replacement.dataset.governanceAvailable = 'false'; state.flush();
      replacement.dataset.governanceAvailable = 'true'; state.flush();
      probe('former focus not restored on availability transition', () => assert.equal(replacement.querySelector('[data-logic-search]').focusCount, 0));
    }
    probe('observer delivery remains bounded', () => assert.ok(state.deliveries < 25));
  }
}
console.log(JSON.stringify({result:'RETAINED_OWNER_IDENTITY_PROOF', observations:identityProof}));
assert.ok(identityProof.every(row => row.result === 'PASS'), 'connected former Governance owner identity must retain denial and revoke old intent');
for (const native of fixtures.filter(row => row.context !== 'healthcare-clinician')) {
  for (const fault of ['removed-source', 'empty-source', 'markerless-replacement', 'removed-subtree', 'mismatch', 'duplicate-before', 'duplicate-after', 'outside-host', 'missing-body']) {
    const state = fixture(native), old = state.root, saved = seed(state), source = old.dataset.governanceSource;
    let invalid = old, duplicate;
    if (fault === 'removed-source') old.removeAttribute('data-governance-source');
    if (fault === 'empty-source') old.dataset.governanceSource = '';
    if (fault === 'markerless-replacement') { invalid = state.attach(); invalid.removeAttribute('data-governance-source'); old.replaceWith(invalid); }
    if (fault === 'removed-subtree') old.remove();
    if (fault === 'mismatch') old.dataset.governanceSource = JSON.stringify(['wrong-context', ...JSON.parse(source).slice(1)]);
    if (fault.startsWith('duplicate')) { duplicate = state.attach(); if (fault === 'duplicate-before') { duplicate.remove(); const index = state.host.childNodes.indexOf(old); duplicate.parentElement = state.host; state.host.childNodes.splice(index, 0, duplicate); } }
    if (fault === 'outside-host') { duplicate = old.cloneNode(true); state.body.append(duplicate); duplicate.remove(); state.body.childNodes.unshift(duplicate); duplicate.parentElement = state.body; }
    if (fault === 'missing-body') old.innerHTML = '';
    state.flush(); state.unavailable([old, invalid, duplicate]);
    for (const control of [saved.route, saved.summary, invalid.querySelector('[data-governance-route]'), ...state.rails()].filter(control => control?.isConnected && (control.closest('#governance-preview') || control.hasAttribute('data-governance-rail')))) state.cancelled(control);
    if (!old.isConnected) assert.equal(state.emit('click', saved.attention).documentDelivered, false, 'removed subtree cannot manufacture document cancellation');
    assert.equal(saved.search.value, 'payroll', 'rejected filters do not apply delegated changes');
    state.emit('click', saved.attention); assert.equal(saved.attention.getAttribute('aria-pressed'), 'true');
    for (const node of [old, invalid, duplicate].filter(Boolean)) node.remove();
    const replacement = state.attach(); replacement.dataset.governanceMode = 'island'; state.root = replacement;
    state.hubOwner.innerHTML = native.hub;
    const hub = state.hubOwner.querySelector('[data-hub-source]'); hub.dataset.hubMode = 'island';
    hub.querySelectorAll('button, textarea').forEach(control => { control.disabled = false; });
    const tab = hub.querySelector('#hub-tab-mail'); const clicks = tab.clickCount;
    state.flush(); state.available(replacement);
    assert.equal(replacement.querySelector('[data-logic-search]').value, '', 'revocation clears old local intent');
    assert.equal(tab.clickCount, clicks, 'restoration must not replay stale pending channel intent');
    assert.ok(replacement.querySelectorAll('[data-governance-focus]').every(node => node.focusCount === 0), 'restoration must not replay revoked focus');
    assert.ok(!state.emit('click', state.unrelated).defaultPrevented, 'unrelated Payroll navigation remains available');
    assert.equal(state.location.href, state.unrelated.href);
    assert.ok(state.deliveries < 20, 'projection settles with bounded observer delivery');
  }
  const absence = fixture(native, false); absence.unavailable([]);
  for (const rail of absence.rails()) absence.cancelled(rail);
  absence.root = absence.attach(); absence.flush(); absence.available();
  const same = fixture(native), search = same.root.querySelector('[data-logic-search]'), source = same.root.dataset.governanceSource;
  search.value = 'payroll'; same.emit('input', search); same.root.dataset.governanceSource = ''; same.flush();
  same.root.dataset.governanceSource = source;
  same.flush(); same.available(); assert.equal(search.value, '', 'same-node source restoration starts clean');
  const guarded = fixture(native), hub = guarded.hubOwner.querySelector('[data-hub-source]'); hub.dataset.hubSource = 'mismatch';
  const channel = guarded.root.querySelector('[data-governance-channel="Mail"]'); assert.ok(guarded.emit('click', channel).defaultPrevented, 'existing Hub source guard preserved');
  const valid = fixture(native), route = valid.root.querySelector('[data-governance-route="risk"]');
  assert.ok(!valid.emit('click', route).defaultPrevented, 'valid existing Governance route remains admitted');
  valid.root.querySelector('[data-logic-search]').focus(); const old = valid.root; old.replaceWith(valid.attach()); valid.root = valid.host.querySelector('#governance-preview'); valid.flush();
  assert.equal(valid.document.activeElement, valid.root.querySelector('[data-logic-search]'), 'valid same-source remount preserves owned focus');
  for (const intent of ['focus', 'navigation']) {
    const newer = fixture(native); newer.root.querySelector('[data-logic-search]').focus();
    newer.root.replaceWith(newer.attach()); newer.root = newer.host.querySelector('#governance-preview');
    if (intent === 'focus') newer.unrelated.focus(); else newer.location.href = 'https://preview.invalid/#payroll-cockpit';
    newer.flush();
    assert.equal(newer.root.querySelector('[data-logic-search]').focusCount, 0, `later ${intent} owns the remount`);
  }
}
const clinician = fixture(fixtures.find(row => row.context === 'healthcare-clinician'));
assert.equal(clinician.root.querySelectorAll('button, input, summary, [data-governance-route]').length, 0, 'native clinician fixture omits executive controls');
clinician.unavailable();
console.log(JSON.stringify({result:'PASS_PRODUCTION_GOVERNANCE_OWNER_LIFETIME', contexts:2, faults:9, browserEfficacy:'Unknown'}));
"##;
