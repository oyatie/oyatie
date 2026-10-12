pub(super) const SCRIPT: &str = r#"
(() => {
  const host = document.getElementById('dashboard-island-root');
  let savedFocus = null;
  let settling = false;
  const owner = () => document.querySelector('#work-hub > .communications-hub[data-hub-source]');
  const binding = current => ({ source: current.dataset.hubSource, context: current.dataset.hubContext });
  const matches = (current, saved) => current?.isConnected && host?.contains(current)
    && saved.source && saved.context && current.dataset.hubSource === saved.source
    && current.dataset.hubContext === saved.context;
  function available(current) {
    const dashboard = current.closest('[data-island="render-envelope-dashboard"]');
    const requested = dashboard?.querySelector('[data-sample-context-requested]');
    const composer = current.querySelector('#hub-composer-input');
    return current.dataset.hubMode === 'island' && dashboard?.getAttribute('aria-busy') === 'false'
      && requested?.dataset.sampleContextRequested === current.dataset.hubContext && composer && !composer.disabled;
  }
  function control(current, target) {
    if (!target?.isConnected || !current?.contains(target)
      || target.closest('.communications-hub[data-hub-source]') !== current || target.disabled) return false;
    if (target.id === 'hub-composer-input') return target.tagName === 'TEXTAREA';
    return ['hub-tab-messenger', 'hub-tab-mail', 'hub-tab-community'].includes(target.id)
      && target.getAttribute('role') === 'tab' && target.parentElement === current.querySelector('.hub-tabs');
  }
  function remember(target, focusTime = savedFocus?.focusTime) {
    const current = owner();
    if (!control(current, target) || !available(current)) return false;
    savedFocus = { ...binding(current), node: target, id: target.id, intentURL: location.href, focusTime };
    return true;
  }
  function handoff(current, target) {
    return savedFocus && !savedFocus.node.isConnected && matches(current, savedFocus)
      && location.href === savedFocus.intentURL && location.hash === '#work-hub'
      && target === current.parentElement && target.id === 'work-hub';
  }
  function restore() {
    if (!savedFocus || savedFocus.node.isConnected) return;
    if (location.href !== savedFocus.intentURL) { savedFocus = null; return; }
    const current = owner();
    if (!current) return;
    if (!matches(current, savedFocus)) { savedFocus = null; return; }
    if (!available(current)) return;
    const active = document.activeElement;
    if (active && active !== document.body && active.isConnected
      && !(active === savedFocus.handoffFocus && handoff(current, active))) {
      savedFocus = null;
      return;
    }
    const previous = savedFocus;
    const target = current.querySelector(`#${previous.id}`);
    if (!control(current, target) || !target.getClientRects().length) { savedFocus = null; return; }
    const start = previous.node.selectionStart, end = previous.node.selectionEnd;
    target.focus({ preventScroll: true });
    if (document.activeElement !== target) { savedFocus = null; return; }
    if (start !== undefined && typeof target.setSelectionRange === 'function') target.setSelectionRange(start, end);
    remember(target, previous.focusTime);
  }
  function synchronize() {
    if (settling) return;
    settling = true;
    queueMicrotask(() => { settling = false; restore(); });
  }
  document.addEventListener('focusin', event => {
    if (!(event.target instanceof Element) || !event.target.isConnected) return;
    if (remember(event.target, event.timeStamp)) return;
    if (handoff(owner(), event.target)) { savedFocus.handoffFocus = event.target; return; }
    if (event.target !== document.body) savedFocus = null;
  });
  document.addEventListener('input', event => {
    if (event.target instanceof Element && event.target === document.activeElement) remember(event.target);
  });
  for (const name of ['pointerdown', 'keydown']) document.addEventListener(name, event => {
    if (!event.isTrusted || !(event.target instanceof Element) || !savedFocus) return;
    if (!savedFocus.node.isConnected || event.target !== savedFocus.node) savedFocus = null;
  }, true);
  window.addEventListener('popstate', () => { savedFocus = null; });
  window.addEventListener('hashchange', event => {
    if (!savedFocus) return;
    const active = document.activeElement;
    const owned = savedFocus.node.isConnected ? active === savedFocus.node
      : !active || active === document.body || !active.isConnected || handoff(owner(), active);
    const earlier = event.isTrusted && owned && Number.isFinite(event.timeStamp)
      && Number.isFinite(savedFocus.focusTime) && event.timeStamp < savedFocus.focusTime
      && event.newURL === savedFocus.intentURL && location.href === event.newURL;
    if (!earlier) savedFocus = null;
  });
  if (host) new MutationObserver(synchronize).observe(host, { childList: true, subtree: true, attributes: true,
    attributeFilter: ['data-hub-source', 'data-hub-context', 'data-hub-mode', 'aria-busy', 'disabled'] });
})();
"#;

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::SCRIPT;

    const LIFETIME_CHECK: &str = r#"
const assert = require('node:assert/strict');
const vm = require('node:vm');
function fixture(fragment = '') {
  const documentEvents = {}, windowEvents = {}, microtasks = [];
  let observer, clock = 0, current;
  const emit = (events, name, event) => (events[name] || []).forEach(callback => callback(event));
  class Element {
    constructor(id, tagName = 'DIV', dataset = {}) {
      this.id = id; this.tagName = tagName; this.dataset = dataset; this.children = [];
      this.isConnected = true; this.disabled = false; this.attrs = {}; this.focusCount = 0; this.visible = true;
    }
    append(child) { child.parentElement = this; this.children.push(child); return child; }
    contains(target) { for (let node = target; node; node = node.parentElement) if (node === this) return true; return false; }
    closest(selector) {
      for (let node = this; node; node = node.parentElement) {
        if (selector === '.communications-hub[data-hub-source]' && 'hubSource' in node.dataset) return node;
        if (selector === '[data-island="render-envelope-dashboard"]' && node.dataset.island === 'render-envelope-dashboard') return node;
      }
      return null;
    }
    querySelector(selector) {
      const matches = node => selector.startsWith('#') ? node.id === selector.slice(1)
        : selector === '.hub-tabs' ? node.id === 'tabs'
        : selector === '[data-sample-context-requested]' && 'sampleContextRequested' in node.dataset;
      const find = node => node.children.map(child => matches(child) ? child : find(child)).find(Boolean);
      return find(this) || null;
    }
    getAttribute(name) { return this.attrs[name] ?? null; }
    getClientRects() { return this.visible ? [{}] : []; }
    focus() { this.focusCount++; document.activeElement = this; emit(documentEvents, 'focusin', { target: this, timeStamp: ++clock }); }
    setSelectionRange(start, end) { this.selectionStart = start; this.selectionEnd = end; }
  }
  const body = new Element('body', 'BODY');
  const host = body.append(new Element('dashboard-island-root'));
  const outside = body.append(new Element('passkey-sign-in', 'BUTTON'));
  const document = { body, activeElement: body,
    getElementById: id => id === host.id ? host : null,
    querySelector: () => current?.hub.isConnected ? current.hub : null,
    addEventListener: (name, callback) => (documentEvents[name] ||= []).push(callback) };
  const url = new URL(`https://preview.invalid/${fragment}`);
  const location = { get href() { return url.href; }, get hash() { return url.hash; } };
  const window = { addEventListener: (name, callback) => (windowEvents[name] ||= []).push(callback) };
  function attach({ source = 'exact-source', context = 'tenant-admin', requested = context, mode = 'island', busy = false } = {}) {
    const dashboard = host.append(new Element('dashboard', 'DIV', { island: 'render-envelope-dashboard' }));
    dashboard.attrs['aria-busy'] = String(busy);
    const request = dashboard.append(new Element('requested', 'SECTION', { sampleContextRequested: requested }));
    const root = dashboard.append(new Element('work-hub', 'SECTION'));
    const hub = root.append(new Element('owner', 'DIV', { hubSource: source, hubContext: context, hubMode: mode }));
    const tabs = hub.append(new Element('tabs'));
    const tab = tabs.append(new Element('hub-tab-mail', 'BUTTON')); tab.attrs.role = 'tab';
    const composer = hub.append(new Element('hub-composer-input', 'TEXTAREA'));
    composer.selectionStart = composer.selectionEnd = 0; composer.disabled = mode !== 'island' || busy;
    const other = hub.append(new Element('preview', 'BUTTON'));
    current = { dashboard, request, root, hub, tab, composer, other };
    observer?.();
    return current;
  }
  function detach() {
    const disconnect = node => { node.isConnected = false; node.children.forEach(disconnect); };
    for (const child of host.children) { disconnect(child); child.parentElement = null; }
    host.children = [];
    if (!document.activeElement.isConnected) document.activeElement = body;
    observer?.();
  }
  function flush() {
    for (let count = 0; microtasks.length; count++) {
      assert.ok(count < 10, 'bounded restoration microtasks'); microtasks.shift()();
    }
  }
  attach();
  vm.runInNewContext(process.argv[1], { document, window, location, Element,
    queueMicrotask: callback => microtasks.push(callback),
    MutationObserver: class { constructor(callback) { observer = callback; } observe() {} },
  }, { timeout: 1000 });
  return { get current() { return current; }, document, outside, attach, detach, flush,
    notify: () => observer(),
    intent: (name, target) => emit(documentEvents, name, { target, isTrusted: true }),
    focusEvent: target => emit(documentEvents, 'focusin', { target, timeStamp: ++clock }),
    popstate: () => emit(windowEvents, 'popstate', { isTrusted: true }),
    hashchange: timeStamp => emit(windowEvents, 'hashchange', { isTrusted: true, timeStamp, newURL: location.href }),
    changeURL: () => { url.hash = 'evidence-spine'; },
  };
}
function seeded(fragment) {
  const state = fixture(fragment);
  state.current.composer.focus();
  state.current.composer.setSelectionRange(2, 8);
  return state;
}

const repeated = seeded();
repeated.current.composer.setSelectionRange(3, 9);
repeated.detach(); repeated.flush();
const first = repeated.attach(); repeated.flush();
assert.equal(repeated.document.activeElement, first.composer, 'matching remount restores composer');
assert.deepEqual([first.composer.selectionStart, first.composer.selectionEnd], [3, 9], 'latest native caret requires no new focus/input');
first.composer.setSelectionRange(1, 5);
repeated.detach(); repeated.flush();
const second = repeated.attach(); repeated.flush();
assert.equal(repeated.document.activeElement, second.composer, 'second remount owns current composer');
assert.deepEqual([second.composer.selectionStart, second.composer.selectionEnd], [1, 5], 'second caret belongs to current old node');
second.tab.focus(); repeated.detach();
const tab = repeated.attach(); repeated.flush();
assert.equal(repeated.document.activeElement, tab.tab, 'controlled Hub tab focus restores without changing selection');

const loading = seeded(); loading.detach();
const waiting = loading.attach({ busy: true }); loading.flush();
assert.equal(waiting.composer.focusCount, 0, 'unavailable replacement cannot receive focus');
waiting.dashboard.attrs['aria-busy'] = 'false'; waiting.composer.disabled = false;
loading.notify(); loading.flush();
assert.equal(loading.document.activeElement, waiting.composer, 'matching availability permits retained restoration');

for (const options of [{ source: 'changed-source' }, { context: 'corporate-office' }, { source: '' }, { requested: 'corporate-office' }, { mode: 'ssr' }]) {
  const state = seeded(); state.detach(); const fresh = state.attach(options); state.flush();
  assert.equal(fresh.composer.focusCount, 0, `changed/unavailable binding refuses focus: ${JSON.stringify(options)}`);
}
for (const field of ['disabled', 'visible']) {
  const state = seeded(); state.detach(); const fresh = state.attach();
  fresh.composer[field] = field === 'disabled'; state.flush();
  assert.equal(fresh.composer.focusCount, 0, `unavailable control refuses focus: ${field}`);
}

for (const name of ['pointerdown', 'keydown']) {
  const state = seeded(); state.detach(); state.intent(name, state.outside);
  const fresh = state.attach(); state.flush();
  assert.equal(fresh.composer.focusCount, 0, `later real ${name} wins during gap`);
}
for (const focusOutside of [true, false]) {
  const state = seeded(); state.detach(); const fresh = state.attach();
  const later = focusOutside ? state.outside : fresh.other; later.focus(); state.flush();
  assert.equal(state.document.activeElement, later, 'connected account/other Hub control focus wins');
  assert.equal(fresh.composer.focusCount, 0, 'restoration never steals newer control focus');
}
for (const cancel of [state => state.popstate(), state => state.changeURL(), state => state.hashchange(1), state => state.hashchange(NaN)]) {
  const state = seeded(); state.detach(); cancel(state);
  const fresh = state.attach(); state.flush();
  assert.equal(fresh.composer.focusCount, 0, 'newer/equal/unknown history or URL intent cancels stale restoration');
}
const earlier = seeded(); earlier.detach(); earlier.hashchange(0);
const earlierFresh = earlier.attach(); earlier.flush();
assert.equal(earlier.document.activeElement, earlierFresh.composer, 'earlier same-URL hash does not erase newer focus');

const handoff = seeded('#work-hub'); handoff.detach();
const handoffFresh = handoff.attach(); handoffFresh.root.focus(); handoff.flush();
assert.equal(handoff.document.activeElement, handoffFresh.composer, 'matching automatic fragment handoff permits original local focus');
const rootIntent = seeded('#work-hub'); rootIntent.detach();
const rootFresh = rootIntent.attach(); rootIntent.intent('pointerdown', rootFresh.root); rootFresh.root.focus(); rootIntent.flush();
assert.equal(rootIntent.document.activeElement, rootFresh.root, 'later real fragment destination intent wins');

const stale = seeded(); const old = stale.current.composer; stale.detach();
const staleFresh = stale.attach(); stale.focusEvent(old); stale.flush();
assert.equal(stale.document.activeElement, staleFresh.composer, 'disconnected stale focus event cannot cancel current handoff');
const ssr = fixture(); ssr.detach(); const staticView = ssr.attach({ mode: 'ssr' });
staticView.root.focus(); ssr.detach(); const native = ssr.attach(); ssr.flush();
assert.equal(native.composer.focusCount, 0, 'SSR destination takeover remains outside local composer restoration');
"#;

    #[test]
    fn hub_focus_lifetime_preserves_latest_caret_and_refuses_newer_or_changed_intent() {
        let output = Command::new("node")
            .args(["-e", LIFETIME_CHECK, SCRIPT])
            .output()
            .expect("node must be available for the Hub focus lifetime regression");
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
