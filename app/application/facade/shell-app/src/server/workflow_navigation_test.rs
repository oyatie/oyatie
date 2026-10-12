use std::process::Command;

use super::SCRIPT;

const HISTORY_CHECK: &str = r#"
const assert = require('node:assert/strict');
const vm = require('node:vm');
function fixture(extraPopstate = false) {
  const documentEvents = {}, windowEvents = {}, frames = [], microtasks = [], hashes = [];
  let observer, clock = 0;
  class Element {
    constructor(id = '', dataset = {}) { this.id = id; this.dataset = dataset; this.children = []; this.isConnected = true; this.focusCount = 0; this.textContent = 'idle'; this.attrs = {}; }
    append(child) { child.parentElement = this; this.children.push(child); return child; }
    contains(target) { for (let node = target; node; node = node.parentElement) if (node === this) return true; return false; }
    closest(selector) {
      const matches = node => selector === '[data-island="render-envelope-dashboard"]' ? node.dataset.island === 'render-envelope-dashboard'
        : selector === '[data-workflow-output-route]' ? 'workflowOutputRoute' in node.dataset
        : selector === '[data-workflow-focus]' ? 'workflowFocus' in node.dataset : false;
      for (let node = this; node; node = node.parentElement) if (matches(node)) return node;
      return null;
    }
    querySelector(selector) { return this.querySelectorAll(selector)[0] || null; }
    querySelectorAll(selector) {
      const key = selector === '[data-workflow-output-status]' ? 'workflowOutputStatus' : 'workflowFocus';
      return this.children.flatMap(child => [...(key in child.dataset ? [child] : []), ...child.querySelectorAll(selector)]);
    }
    hasAttribute(name) { return name in this.attrs; }
    setAttribute(name, value) { this.attrs[name] = value; }
    scrollIntoView() { this.scrollCount = (this.scrollCount || 0) + 1; }
    focus() { this.focusCount++; document.activeElement = this; emit(documentEvents, 'focusin', { target: this, timeStamp: ++clock }); }
  }
  const body = new Element('body');
  const host = body.append(new Element('dashboard-island-root', { island: 'render-envelope-dashboard' }));
  const later = host.append(new Element('context-title'));
  const find = (node, id) => node.id === id ? node : node.children.map(child => find(child, id)).find(Boolean);
  const emit = (events, name, event) => (events[name] || []).forEach(callback => callback(event));
  const document = { body, activeElement: body, getElementById: id => find(body, id), addEventListener: (name, callback) => (documentEvents[name] ||= []).push(callback) };
  const url = new URL('https://preview.invalid/#workflow-studio');
  const location = {
    get href() { return url.href; }, get hash() { return url.hash; },
    set hash(value) {
      const oldURL = url.href;
      url.hash = value;
      emit(windowEvents, 'popstate', { isTrusted: true });
      if (extraPopstate) emit(windowEvents, 'popstate', { isTrusted: true });
      hashes.push({ oldURL, newURL: url.href, isTrusted: true, timeStamp: ++clock });
    },
  };
  const window = { addEventListener: (name, callback) => (windowEvents[name] ||= []).push(callback) };
  let current;
  function detach() {
    const disconnect = node => { node.isConnected = false; node.children.forEach(disconnect); };
    for (const child of host.children.filter(child => child !== later)) { disconnect(child); child.parentElement = null; }
    host.children = [later];
    if (!document.activeElement.isConnected) document.activeElement = body;
    observer?.();
  }
  function attach(mode) {
    const panel = host.append(new Element('workflow-studio', { workflowSource: 'complete-source', workflowContext: 'tenant-admin', workflowMode: mode, workflowFocus: 'panel' }));
    const control = panel.append(new Element('', { workflowOutputRoute: 'evidence' }));
    const field = panel.append(new Element('', { workflowFocus: 'selected-kind' }));
    field.selectionStart = 0;
    field.selectionEnd = 0;
    field.setSelectionRange = function (start, end) { this.selectionStart = start; this.selectionEnd = end; };
    const status = panel.append(new Element('', { workflowOutputStatus: 'true' }));
    const evidence = host.append(new Element('evidence-spine'));
    current = { panel, control, field, status, evidence };
    observer?.();
    return current;
  }
  attach('ssr');
  vm.runInNewContext(process.argv[1], {
    document, window, location, Element, URL,
    requestAnimationFrame: callback => frames.push(callback), queueMicrotask: callback => microtasks.push(callback),
    MutationObserver: class { constructor(callback) { observer = callback; } observe() {} },
  }, { timeout: 1000 });
  function drain(queue) { for (let count = 0; queue.length; count++) { assert.ok(count < 20, 'bounded callback drain'); queue.shift()(); } }
  return {
    get current() { return current; }, document, location, later,
    route: () => emit(documentEvents, 'click', { target: current.control, button: 0 }),
    frame: () => drain(frames), microtask: () => drain(microtasks),
    hash: () => hashes.splice(0).forEach(event => emit(windowEvents, 'hashchange', event)),
    externalPopstate: () => emit(windowEvents, 'popstate', { isTrusted: true }),
    externalHashEvent: event => emit(windowEvents, 'hashchange', event),
    externalHash: value => { location.hash = value; }, detach, attach,
    trustedIntent: (name, target) => emit(documentEvents, name, { target, isTrusted: true }),
  };
}

const own = fixture();
own.route(); own.frame();
assert.equal(own.location.hash, '#evidence-spine', 'confirmed route keeps its own fragment');
assert.equal(own.document.activeElement, own.current.evidence, 'synchronous own popstate permits confirmed focus');
own.hash(); own.detach(); own.microtask();
const native = own.attach('island'); own.microtask(); own.frame();
assert.equal(own.document.activeElement, native.evidence, 'owned SSR handoff survives matching island takeover');
assert.equal(native.evidence.focusCount, 1, 'one native handoff consumes the route');

const second = fixture(true);
second.route(); second.frame();
assert.equal(second.current.evidence.focusCount, 0, 'second synchronous popstate is later intent');
assert.equal(second.current.status.textContent, 'idle', 'canceled route cannot report confirmed success');

const later = fixture();
later.route(); later.frame();
later.externalPopstate(); later.hash(); later.detach(); later.microtask();
const canceled = later.attach('island'); later.microtask(); later.frame();
assert.equal(canceled.evidence.focusCount, 0, 'later same-URL history cancels pending handoff before owned hashchange');
assert.equal(canceled.status.textContent, 'idle', 'old hashchange cannot revive a canceled route');

const queued = fixture();
queued.route(); queued.externalHash('#context-title'); queued.frame(); queued.hash();
assert.equal(queued.location.hash, '#context-title', 'later fragment wins before queued route completion');
assert.equal(queued.current.evidence.focusCount, 0, 'queued stale route cannot focus its target');

const focus = fixture();
focus.current.field.focus(); focus.detach(); focus.microtask();
focus.externalPopstate(); focus.later.focus();
const replacement = focus.attach('island'); focus.microtask(); focus.frame();
assert.equal(focus.document.activeElement, focus.later, 'later same-URL history wins while old editor focus is detached');
assert.equal(replacement.field.focusCount, 0, 'canceled detached focus cannot restore Workflow');

function seededFocus() {
  const state = fixture();
  state.externalHash('#evidence-spine');
  state.current.field.focus();
  return state;
}
const earlier = seededFocus();
earlier.hash(); earlier.detach(); earlier.microtask();
const retained = earlier.attach('island'); earlier.microtask();
assert.equal(earlier.document.activeElement, retained.field, 'older native hash completion preserves the newer bound editor focus');

const gap = seededFocus();
gap.detach(); gap.hash(); gap.microtask();
const afterGap = gap.attach('island'); gap.microtask();
assert.equal(gap.document.activeElement, afterGap.field, 'older hash completion during loading preserves detached editor intent for matching restoration');

const laterGap = seededFocus();
laterGap.detach(); laterGap.externalPopstate(); laterGap.hash(); laterGap.microtask();
const canceledGap = laterGap.attach('island'); laterGap.microtask();
assert.equal(canceledGap.field.focusCount, 0, 'later same-URL popstate still cancels the detached editor token before old hash delivery');

const changedGap = seededFocus();
changedGap.detach(); changedGap.hash(); changedGap.microtask();
const changedFresh = changedGap.attach('island'); changedFresh.panel.dataset.workflowContext = 'tenant-viewer'; changedGap.microtask();
assert.equal(changedFresh.field.focusCount, 0, 'retained detached focus is refused at restoration when current context changes');

for (const timeStamp of [2, 3, NaN]) {
  const laterHash = seededFocus();
  laterHash.externalHashEvent({ isTrusted: true, timeStamp, oldURL: 'https://preview.invalid/#workflow-studio', newURL: laterHash.location.href });
  laterHash.detach(); laterHash.microtask();
  const next = laterHash.attach('island'); laterHash.microtask();
  assert.equal(next.field.focusCount, 0, 'equal, newer or unknown history time cancels editor intent');
}
const changed = seededFocus();
changed.current.panel.dataset.workflowContext = 'tenant-viewer';
changed.hash(); changed.detach(); changed.microtask();
const reset = changed.attach('island'); changed.microtask();
assert.equal(reset.field.focusCount, 0, 'earlier history cannot import a focus token across changed context');

const caret = fixture();
caret.current.field.focus();
caret.current.field.setSelectionRange(1, 4);
caret.detach(); caret.microtask();
const firstCaret = caret.attach('island'); caret.microtask();
assert.equal(caret.document.activeElement, firstCaret.field, 'matching replacement owns the current text focus');
assert.deepEqual([firstCaret.field.selectionStart, firstCaret.field.selectionEnd], [1, 4], 'matching replacement retains latest native caret changed without focus or input');
firstCaret.field.setSelectionRange(2, 5);
caret.detach(); caret.microtask();
const secondCaret = caret.attach('island'); caret.microtask();
assert.equal(caret.document.activeElement, secondCaret.field, 'second current-round replacement owns the current text focus');
assert.deepEqual([secondCaret.field.selectionStart, secondCaret.field.selectionEnd], [2, 5], 'second replacement retains its current native caret instead of an earlier snapshot');


function replacementPanelFocus() {
  const state = fixture();
  state.current.field.dataset.workflowFocus = 'selected-label';
  state.current.field.focus();
  state.detach(); state.microtask();
  const fresh = state.attach('island');
  fresh.field.dataset.workflowFocus = 'selected-label';
  // The settled render focuses this fragment root before MutationObserver replay.
  fresh.panel.focus();
  return { state, fresh };
}
const temporaryPanel = replacementPanelFocus();
const activeBeforeHR = temporaryPanel.state.document.activeElement;
temporaryPanel.state.later.id = 'identity-tab-employees';
temporaryPanel.state.later.focus();
if (activeBeforeHR.isConnected) activeBeforeHR.focus();
temporaryPanel.state.microtask();
assert.equal(temporaryPanel.state.document.activeElement, temporaryPanel.fresh.field,
  'automatic matching panel and temporary HR focus preserve detached selected-label for observer restoration');
assert.equal(temporaryPanel.fresh.field.focusCount, 1, 'one matching restoration consumes the detached field anchor');

for (const name of ['pointerdown', 'keydown']) {
  const newer = replacementPanelFocus();
  newer.state.trustedIntent(name, newer.fresh.panel);
  newer.state.microtask();
  assert.equal(newer.state.document.activeElement, newer.fresh.panel, `later trusted ${name} inside fresh Workflow wins`);
  assert.equal(newer.fresh.field.focusCount, 0, `later trusted ${name} cancels detached field restoration`);
}
const outsidePanel = replacementPanelFocus();
const outsideTarget = outsidePanel.state.document.body.append(new outsidePanel.state.later.constructor('outside-dashboard'));
assert.equal(outsidePanel.state.document.getElementById('dashboard-island-root').contains(outsideTarget), false,
  'outside-focus fixture target is connected outside the dashboard host');
assert.equal(outsideTarget.isConnected, true, 'outside-focus fixture target is connected');
outsideTarget.focus(); outsidePanel.state.microtask();
assert.equal(outsidePanel.state.document.activeElement, outsideTarget, 'later outside-host focus wins over temporary panel focus');
assert.equal(outsidePanel.fresh.field.focusCount, 0, 'outside focus cancels detached selected-label restoration');
for (const key of ['workflowSource', 'workflowContext']) {
  const mismatch = replacementPanelFocus();
  mismatch.fresh.panel.dataset[key] = 'different-current-binding';
  mismatch.state.microtask();
  assert.equal(mismatch.fresh.field.focusCount, 0, `fresh panel focus never imports selected-label across changed ${key}`);
}

console.log('Workflow owned fragment and later history regression PASS');
"#;

#[test]
fn workflow_navigation_owned_fragment_preserves_handoff_and_later_history_wins() {
    let output = Command::new("node")
        .args(["-e", HISTORY_CHECK, SCRIPT])
        .output()
        .expect("Node runs the native Workflow bridge regression");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
