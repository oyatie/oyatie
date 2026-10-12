use std::io::Write;
use std::process::{Command, Stdio};

const OBSERVER_CHECK: &str = r##"
const assert = require('node:assert/strict'), vm = require('node:vm');
const html = require('node:fs').readFileSync(0, 'utf8');
const events = {}, microtasks = [], mutations = [], delivered = [], focuses = [];
let observer, observedRoot, options, effects = 0;
function changed(target, attributeName, type = 'attributes') {
  if (!observer || !observedRoot.contains(target)) return;
  if (target !== observedRoot && !options.subtree) return;
  if (type === 'attributes' ? options.attributes && options.attributeFilter.includes(attributeName) : options[type])
    mutations.push({target, attributeName, type});
}
const decode = text => text.replace(/&(quot|apos|amp|lt|gt|#x[0-9a-f]+|#[0-9]+);/gi, (_, name) => {
  const named = {quot: '"', apos: "'", amp: '&', lt: '<', gt: '>'};
  return name[0] === '#' ? String.fromCodePoint(name[1].toLowerCase() === 'x' ? parseInt(name.slice(2), 16) : Number(name.slice(1))) : named[name.toLowerCase()];
});
class Element {
  constructor(tag, attrs = {}) { this.tagName = tag.toUpperCase(); this.attrs = attrs; this.children = []; this.parts = []; this.value = attrs.value || ''; this.scrollLeft = 0; this.selectionStart = 0; this.selectionEnd = 0; }
  get id() { return this.getAttribute('id') || ''; }
  get dataset() { return Object.fromEntries(Object.entries(this.attrs).filter(([name]) => name.startsWith('data-')).map(([name, value]) => [name.slice(5).replace(/-([a-z])/g, (_, letter) => letter.toUpperCase()), value])); }
  get isConnected() { return this === tree || !!this.parentElement?.isConnected; }
  get childElementCount() { return this.children.length; }
  get nextElementSibling() { const siblings = this.parentElement?.children || []; return siblings[siblings.indexOf(this) + 1] || null; }
  get disabled() { return this.hasAttribute('disabled'); }
  set disabled(value) { if (value) this.setAttribute('disabled', ''); else this.removeAttribute('disabled'); }
  get hidden() { return this.hasAttribute('hidden'); }
  set hidden(value) { if (value) this.setAttribute('hidden', ''); else this.removeAttribute('hidden'); }
  get textContent() { return this.parts.map(part => typeof part === 'string' ? part : part.textContent).join(''); }
  set textContent(value) { this.children.forEach(child => child.parentElement = null); this.children = []; this.parts = [String(value)]; changed(this, null, 'childList'); }
  get classList() {
    const names = new Set((this.getAttribute('class') || '').split(/\s+/).filter(Boolean));
    return {contains: name => names.has(name), toggle: (name, selected) => {
      const before = names.has(name); if (selected) names.add(name); else names.delete(name);
      if (before !== selected) this.setAttribute('class', [...names].join(' '));
    }};
  }
  getAttribute(name) { return this.attrs[name] ?? null; }
  hasAttribute(name) { return this.getAttribute(name) !== null; }
  setAttribute(name, value) { this.attrs[name] = String(value); changed(this, name); }
  removeAttribute(name) { if (this.hasAttribute(name)) { delete this.attrs[name]; changed(this, name); } }
  append(child) { child.parentElement = this; this.children.push(child); this.parts.push(child); return child; }
  contains(target) { for (let node = target; node; node = node.parentElement) if (node === this) return true; return false; }
  matcher(selector) {
    const alternatives = selector.split(',').map(part => {
      part = part.trim(); if (part === '*') return () => true;
      const attributes = [...part.matchAll(/\[([\w-]+)(?:="([^"]*)")?\]/g)];
      const plain = part.replace(/\[([\w-]+)(?:="([^"]*)")?\]/g, '');
      assert.match(plain, /^(?:[a-z][a-z0-9-]*)?(?:[.#][\w-]+)*$/, 'fixture refuses unsupported selectors');
      const tag = plain.match(/^[a-z][a-z0-9-]*/)?.[0].toUpperCase();
      const ids = [...plain.matchAll(/#([\w-]+)/g)].map(match => match[1]);
      const classes = [...plain.matchAll(/\.([\w-]+)/g)].map(match => match[1]);
      return node => (!tag || node.tagName === tag) && ids.every(id => node.id === id)
        && classes.every(name => node.classList.contains(name))
        && attributes.every(([, name, value]) => value === undefined ? node.hasAttribute(name) : node.getAttribute(name) === value);
    });
    return node => alternatives.some(matches => matches(node));
  }
  matches(selector) { return this.matcher(selector)(this); }
  closest(selector) { const matches = this.matcher(selector); for (let node = this; node; node = node.parentElement) if (matches(node)) return node; return null; }
  querySelectorAll(selector) {
    const matches = this.matcher(selector), found = [];
    const visit = node => { for (const child of node.children) { if (matches(child)) found.push(child); visit(child); } };
    visit(this); return found;
  }
  querySelector(selector) {
    const matches = this.matcher(selector);
    const visit = node => {
      for (const child of node.children) { if (matches(child)) return child; const found = visit(child); if (found) return found; }
      return null;
    };
    return visit(this);
  }
  getClientRects() { return this.isConnected && !this.closest('[hidden]') ? [{}] : []; }
  setSelectionRange(start, end) { this.selectionStart = start; this.selectionEnd = end; }
  focus() { focuses.push(this); document.activeElement = this; emit('focusin', {target: this}); }
  click() { effects++; throw new Error('observer recovery must not invoke a native tab or channel callback'); }
}
(() => {
  const attrs = {id: 'duplicate', role: 'tab', class: 'match', 'data-key': 'live'};
  const root = new Element('div', {...attrs}), branch = root.append(new Element('section'));
  const first = branch.append(new Element('span', {...attrs})), second = root.append(new Element('span', {...attrs}));
  const selector = '#duplicate.match[role="tab"][data-key="live"]';
  assert.deepEqual(root.querySelectorAll('#duplicate, .match, span[role="tab"]'), [first, second], 'preorder descendants, duplicates and overlapping alternatives');
  assert.equal(root.querySelector('*'), branch); assert.equal(root.querySelector(selector), first);
  assert.deepEqual(branch.querySelectorAll(selector), [first]); assert.equal(first.querySelector(selector), null);
  assert.equal(first.matches(selector), true); assert.equal(first.closest('section'), branch);
  branch.setAttribute('id', 'duplicate');
  assert.throws(() => root.querySelector('#duplicate, span > strong'), /fixture refuses unsupported selectors/);
  branch.removeAttribute('id');
  for (const [name, value] of Object.entries({id: 'changed', role: 'presentation', class: 'other', 'data-key': 'changed'})) first.setAttribute(name, value);
  for (const query of ['#duplicate', '[role="tab"]', '.match', '[data-key="live"]']) assert.deepEqual(root.querySelectorAll(query), [second], 'queries read current ' + query);
  assert.equal(root.querySelector(selector), second); assert.equal(first.matches(selector), false);
  for (const [name, value] of Object.entries(attrs)) first.setAttribute(name, value);
  assert.equal(root.querySelector(selector), first); assert.equal(first.matches(selector), true);
})();
// Only passive native SSR elements are read. Script and template bodies stay inert.
const passive = html.replace(/<(script|style|template)\b[^>]*>[\s\S]*?<\/\1\s*>/gi, '');
const tree = new Element('document'), stack = [tree];
const voidTags = new Set(['area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input', 'link', 'meta', 'param', 'source', 'track', 'wbr']);
for (const match of passive.matchAll(/<!--[\s\S]*?-->|<![^>]*>|<\/[a-z][^>]*>|<[a-z][^>]*>|[^<]+/gi)) {
  const token = match[0];
  if (token.startsWith('<!')) continue;
  if (token.startsWith('</')) {
    const tag = token.match(/^<\/([a-z][\w-]*)/i)[1].toUpperCase();
    assert.equal(stack.pop().tagName, tag, 'native SSR closes the current element'); continue;
  }
  if (!token.startsWith('<')) { stack.at(-1).parts.push(decode(token)); continue; }
  const tag = token.match(/^<([a-z][\w-]*)/i)[1].toLowerCase(), attrs = {};
  for (const [, name, double, single, bare] of token.slice(tag.length + 1, -1).matchAll(/([^\s=/>]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+)))?/g))
    attrs[name] = decode(double ?? single ?? bare ?? '');
  const element = stack.at(-1).append(new Element(tag, attrs));
  if (!voidTags.has(tag) && !token.endsWith('/>')) stack.push(element);
}
assert.equal(stack.length, 1, 'complete passive SSR tree');
assert.equal(tree.querySelectorAll('script,style,template').length, 0, 'native script and template contents are inert');
const document = {body: tree.querySelector('body'), activeElement: null, hasFocus: () => true,
  getElementById: id => tree.querySelector('#' + id), querySelectorAll: selector => tree.querySelectorAll(selector),
  addEventListener: (name, callback) => (events[name] ||= []).push(callback)};
document.activeElement = document.body;
const emit = (name, event) => (events[name] || []).forEach(callback => callback(event));
const location = new URL('https://preview.invalid/#service-catalog');
vm.runInNewContext(process.argv[1], {document, location, Element,
  window: {addEventListener: (name, callback) => (events[name] ||= []).push(callback)}, queueMicrotask: callback => microtasks.push(callback),
  MutationObserver: class {
    constructor(callback) { observer = callback; }
    observe(root, config) { observedRoot = root; options = config; }
  },
  requestAnimationFrame() { effects++; throw new Error('no route report scheduled by relation recovery'); },
  setTimeout() { effects++; throw new Error('no route timeout scheduled by relation recovery'); },
  fetch() { effects++; throw new Error('no network effect allowed'); },
}, {timeout: 1000});
function drain() {
  for (let count = 0; mutations.length || microtasks.length; count++) {
    assert.ok(count < 30, 'filtered observer and microtask queues settle without loops');
    if (mutations.length) { const batch = mutations.splice(0); delivered.push(...batch); observer(batch); }
    for (const callback of microtasks.splice(0)) callback();
  }
}
drain();
const root = document.getElementById('service-catalog'), search = root.querySelector('[data-catalog-search]');
const tab = document.getElementById('resource-tab-audit'), panel = document.getElementById('resource-panel-audit');
const group = tab.closest('[role="tablist"]'), link = root.querySelector('[data-catalog-link="audit"]');
const reason = document.getElementById('catalog-link-unavailable-audit');
assert.equal(search.disabled, false, 'actual rendered native Catalog passes production admission');
assert.equal(root.dataset.catalogAdmitted, 'true'); assert.equal(root.dataset.catalogAvailable, 'true');
assert.equal(root.dataset.catalogMode, 'ssr');
assert.equal(tab.getAttribute('role'), 'tab'); assert.equal(group.getAttribute('role'), 'tablist');
assert.equal(panel.getAttribute('role'), 'tabpanel'); assert.equal(panel.getAttribute('aria-labelledby'), tab.id);
assert.equal(tab.getAttribute('aria-controls'), panel.id);
search.focus(); search.value = 'audit'; search.setSelectionRange(1, 3); emit('input', {target: search}); drain();
const pin = root.querySelector('[data-catalog-pin="audit-chain"]');
emit('click', {target: pin, button: 0, preventDefault() {}, stopImmediatePropagation() {}}); drain();
const table = root.querySelector('.catalog-table-shell'); table.scrollLeft = 29; emit('scroll', {target: table});
function localModel(catalog = root) {
  const input = catalog.querySelector('[data-catalog-search]');
  return {source: catalog.dataset.catalogSource, query: input.value, selection: [input.selectionStart, input.selectionEnd],
    filters: catalog.querySelectorAll('[data-catalog-filter]').map(node => [node.dataset.catalogFilter, node.getAttribute('aria-pressed')]),
    attention: catalog.querySelector('[data-catalog-attention]').getAttribute('aria-pressed'),
    rows: catalog.querySelectorAll('[data-catalog-module]').map(row => [row.dataset.catalogModule, row.hidden, row.querySelector('[data-catalog-pin]').getAttribute('aria-pressed'), row.querySelector('[data-catalog-detail]').getAttribute('aria-expanded')]),
    inspection: catalog.querySelector('[data-catalog-inspection]').hidden, scroll: catalog.querySelector('.catalog-table-shell').scrollLeft,
    routeStatus: catalog.querySelector('[data-catalog-route-status]').textContent};
}
const before = localModel(), focus = document.activeElement, url = location.href;
assert.equal(before.query, 'audit'); assert.equal(pin.getAttribute('aria-pressed'), 'true');
function presentation(available, label) {
  assert.equal(link.getAttribute('aria-disabled'), String(!available), label + ' route availability');
  assert.equal(link.getAttribute('tabindex'), available ? '0' : '-1', label + ' tab stop');
  assert.equal(reason.hidden, available, label + ' reason visibility');
  assert.equal(reason.textContent, available ? '' : 'Audit sample destination unavailable in this render.', label + ' reason text');
  assert.equal(link.getAttribute('aria-describedby'), available ? 'catalog-availability' : 'catalog-availability ' + reason.id, label + ' description');
  assert.deepEqual(localModel(), before, label + ' preserves the current native Catalog local model');
  assert.equal(document.activeElement, focus, label + ' preserves focus');
  assert.equal(location.href, url, label + ' preserves URL'); assert.equal(effects, 0, label + ' creates no effects');
}
presentation(true, 'initial');
for (const [target, name, invalid] of [[tab, 'aria-controls', 'diagnostic-foreign-panel'],
  [panel, 'aria-labelledby', 'diagnostic-foreign-tab'], [tab, 'role', 'presentation'], [group, 'role', 'presentation']]) {
  const saved = target.getAttribute(name), start = delivered.length;
  target.setAttribute(name, invalid); drain();
  assert.ok(delivered.slice(start).some(record => record.target === target && record.attributeName === name), name + ' invalid mutation is delivered by production observer subscription');
  presentation(false, name + ' invalid');
  const restoreStart = delivered.length;
  target.setAttribute(name, saved); drain();
  assert.ok(delivered.slice(restoreStart).some(record => record.target === target && record.attributeName === name), name + ' restored mutation is delivered by production observer subscription');
  presentation(true, name + ' restored');
  console.log(`Catalog ${target.id || 'native group'} ${name} observer recovery PASS`);
}
function clone(node) {
  const copy = new Element(node.tagName.toLowerCase(), {...node.attrs});
  for (const part of node.parts) typeof part === 'string' ? copy.parts.push(part) : copy.append(clone(part));
  return copy;
}
const host = document.getElementById('dashboard-island-root');
const baseline = clone(root.closest('[data-island="render-envelope-dashboard"]'));
function mount(dashboard) {
  host.children.forEach(child => child.parentElement = null);
  host.children = []; host.parts = [];
  if (dashboard) host.append(dashboard);
  changed(host, null, 'childList'); document.activeElement = document.body;
}
const click = (target, isTrusted = true) => emit('click', {target, isTrusted, button: 0, preventDefault() {}, stopImmediatePropagation() {}});
const failures = [];
for (const scenario of ['no-click', 'automatic-outside-click', 'no-click-gap', 'automatic-gap-click',
  'trusted-outside-click', 'trusted-gap-click', 'trusted-nonfocusable-click', 'outside-focus',
  'changed-url', 'connected-active-element', 'changed-source', 'unavailable-render', 'document-blurred']) {
  try {
  location.href = url; document.hasFocus = () => true;
  const live = clone(baseline); mount(live); drain();
  const previous = live.querySelector('#service-catalog'), input = previous.querySelector('[data-catalog-search]');
  input.focus(); input.value = 'audit'; input.setSelectionRange(1, 3); emit('input', {target: input}); drain();
  const auditPin = previous.querySelector('[data-catalog-pin="audit-chain"]');
  if (auditPin.getAttribute('aria-pressed') !== 'true') click(auditPin);
  click(previous.querySelector(`[data-catalog-filter="${auditPin.closest('[data-catalog-module]').dataset.catalogGroup}"]`));
  click(previous.querySelector('[data-catalog-detail="audit-chain"]')); drain();
  const oldTable = previous.querySelector('.catalog-table-shell'); oldTable.scrollLeft = 29; emit('scroll', {target: oldTable});
  const expected = localModel(previous); delete expected.selection;
  assert.equal(expected.inspection, false, scenario + ' seeds local inspection');
  const replacement = clone(live), next = replacement.querySelector('#service-catalog');
  next.setAttribute('data-catalog-mode', 'island'); replacement.setAttribute('data-fragment-navigation', 'island');
  if (scenario.endsWith('-gap') || scenario.includes('-gap-')) { mount(null); drain(); assert.equal(host.childElementCount, 0); }
  mount(replacement);
  const outside = replacement.querySelector('#resource-tab-audit'), nextInput = next.querySelector('[data-catalog-search]');
  assert.ok(!input.isConnected && next.isConnected && outside.isConnected && !outside.closest('#service-catalog'));
  assert.equal(next.dataset.catalogSource, previous.dataset.catalogSource); assert.equal(document.activeElement, document.body);
  assert.ok(mutations.length > 0 && microtasks.length === 0, scenario + ' precedes replacement synchronization');
  if (!scenario.startsWith('no-click')) click(scenario === 'trusted-nonfocusable-click' ? replacement : outside, scenario.startsWith('trusted-'));
  if (scenario === 'outside-focus') outside.focus();
  if (scenario === 'connected-active-element') document.activeElement = outside;
  if (scenario === 'changed-url') { location.hash = '#resource-panel-audit'; emit('hashchange', {}); }
  if (scenario === 'changed-source') {
    const source = JSON.parse(next.dataset.catalogSource); source[1] += ' different source';
    next.setAttribute('data-catalog-source', JSON.stringify(source)); next.setAttribute('data-catalog-label', source[1]);
  }
  if (scenario === 'unavailable-render') next.setAttribute('data-catalog-available', 'false');
  if (scenario === 'document-blurred') document.hasFocus = () => false;
  const focusStart = focuses.length; drain();
  const restore = scenario.startsWith('no-click') || scenario.startsWith('automatic-');
  assert.equal(focuses.length - focusStart, restore ? 1 : 0, scenario + ' restores focus exactly once or respects the fence');
  assert.equal(document.activeElement, restore ? nextInput : ['outside-focus', 'connected-active-element'].includes(scenario) ? outside : document.body, scenario + ' focus outcome');
  if (restore) assert.deepEqual([nextInput.selectionStart, nextInput.selectionEnd], [1, 3], scenario + ' saved caret');
  const actual = localModel(next); delete actual.selection;
  if (['changed-source', 'unavailable-render'].includes(scenario)) {
    assert.equal(actual.query, ''); assert.equal(actual.inspection, true); assert.equal(actual.scroll, 0);
    assert.ok(actual.rows.every(row => row[2] === 'false')); assert.equal(nextInput.disabled, scenario === 'unavailable-render');
  } else assert.deepEqual(actual, expected, scenario + ' preserves query, category, attention, pins, inspection and scroll');
  assert.equal(effects, 0, scenario + ' has no native callback, route or network effect');
  console.log(`Catalog ${scenario} focus intent PASS`);
  } catch (error) {
    failures.push({scenario, message: error.message});
    console.log(`Catalog ${scenario} focus intent FAIL`);
  }
}
assert.deepEqual(failures, [], 'all production Catalog focus intent and user-intent fence cases');
"##;

#[test]
fn catalog_native_tab_relations_recover_through_filtered_observer() {
    let html = super::rendered_server();
    let mut child = Command::new("node")
        .args(["-e", OBSERVER_CHECK, super::super::script().as_str()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node runs the production Catalog observer regression");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(html.as_bytes())
        .expect("Native rendered SSR enters the observer fixture");
    let output = child
        .wait_with_output()
        .expect("Catalog observer completes");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
