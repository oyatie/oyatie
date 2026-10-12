use std::io::Write;
use std::process::{Command, Stdio};

use leptos::prelude::*;

const DETACHED_ROUTE_CHECK: &str = r##"
const assert = require('node:assert/strict'), vm = require('node:vm');
const html = require('node:fs').readFileSync(0, 'utf8');
const documentEvents = {}, microtasks = [], frames = [];
let observer;
const decode = text => text.replace(/&(quot|apos|amp|lt|gt|#x[0-9a-f]+|#[0-9]+);/gi, (_, name) => {
  const named = {quot: '"', apos: "'", amp: '&', lt: '<', gt: '>'};
  return name[0] === '#' ? String.fromCodePoint(name[1].toLowerCase() === 'x' ? parseInt(name.slice(2), 16) : Number(name.slice(1))) : named[name.toLowerCase()];
});
class Element {
  constructor(tag, attrs = {}) { this.tagName = tag.toUpperCase(); this.attrs = attrs; this.children = []; this.parts = []; this.events = {}; }
  get id() { return this.getAttribute('id') || ''; }
  get dataset() { return Object.fromEntries(Object.entries(this.attrs).filter(([name]) => name.startsWith('data-')).map(([name, value]) => [name.slice(5).replace(/-([a-z])/g, (_, letter) => letter.toUpperCase()), value])); }
  get isConnected() { return this === tree || !!this.parentElement?.isConnected; }
  get childElementCount() { return this.children.length; }
  get disabled() { return this.hasAttribute('disabled'); }
  set disabled(value) { if (value) this.setAttribute('disabled', ''); else this.removeAttribute('disabled'); }
  get hidden() { return this.hasAttribute('hidden'); }
  set hidden(value) { if (value) this.setAttribute('hidden', ''); else this.removeAttribute('hidden'); }
  get textContent() { return this.parts.map(part => typeof part === 'string' ? part : part.textContent).join(''); }
  set textContent(value) { this.children.forEach(child => child.parentElement = null); this.children = []; this.parts = [String(value)]; }
  get classList() { return {contains: name => (this.getAttribute('class') || '').split(/\s+/).includes(name)}; }
  get href() { return new URL(this.getAttribute('href'), location).href; }
  getAttribute(name) { return this.attrs[name] ?? null; }
  hasAttribute(name) { return this.getAttribute(name) !== null; }
  setAttribute(name, value) { this.attrs[name] = String(value); }
  removeAttribute(name) { delete this.attrs[name]; }
  append(child) { child.parentElement = this; this.children.push(child); this.parts.push(child); return child; }
  remove() {
    const parent = this.parentElement;
    parent.children = parent.children.filter(child => child !== this);
    parent.parts = parent.parts.filter(part => part !== this); this.parentElement = null;
  }
  contains(target) { for (let node = target; node; node = node.parentElement) if (node === this) return true; return false; }
  matcher(selector) {
    const alternatives = selector.split(',').map(part => {
      part = part.trim();
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
    if (selector === ':scope > summary') return this.children.filter(child => child.tagName === 'SUMMARY');
    const matches = this.matcher(selector), found = [];
    const visit = node => { for (const child of node.children) { if (matches(child)) found.push(child); visit(child); } };
    visit(this); return found;
  }
  querySelector(selector) { return this.querySelectorAll(selector)[0] || null; }
  getClientRects() { return this.isConnected && !this.closest('[hidden]') ? [{}] : []; }
  addEventListener(name, callback) { (this.events[name] ||= []).push(callback); }
  focus() { document.activeElement = this; emit('focusin', {target: this}); }
}
const tree = new Element('body'), stack = [tree];
const passive = html.replace(/<(script|style|template)\b[^>]*>[\s\S]*?<\/\1\s*>/gi, '');
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
assert.equal(stack.length, 1, 'complete passive native SSR tree');
assert.equal(tree.querySelectorAll('script,style,template').length, 0, 'templates remain inert');
const document = {body: tree, activeElement: tree, hasFocus: () => true,
  getElementById: id => tree.querySelector('#' + id), querySelectorAll: selector => tree.querySelectorAll(selector),
  addEventListener: (name, callback) => (documentEvents[name] ||= []).push(callback)};
const emit = (name, event) => (documentEvents[name] || []).forEach(callback => callback(event));
const location = new URL('https://preview.invalid/#intelligence-command-console');
const root = document.getElementById('intelligence-command-console');
const anchor = root.querySelector('[data-intelligence-route="tenant-workflow-delegation"]');
const authored = [anchor.getAttribute('href'), anchor.dataset.intelligenceRoute, anchor.dataset.intelligenceChannel, root.dataset.intelligenceSource];
const parent = root.parentElement;
function clone(node) {
  const copy = new Element(node.tagName, {...node.attrs});
  for (const part of node.parts) typeof part === 'string' ? copy.parts.push(part) : copy.append(clone(part));
  return copy;
}
const fresh = clone(root);
root.setAttribute('data-intelligence-admitted', 'false');
vm.runInNewContext(process.argv[1], {document, location, Element,
  window: {addEventListener() {}}, performance: {now: () => 0},
  queueMicrotask: callback => microtasks.push(callback), requestAnimationFrame: callback => frames.push(callback), setTimeout() {},
  MutationObserver: class { constructor(callback) { observer = callback; } observe() {} },
}, {timeout: 1000});
assert.equal(anchor.events.click?.length || 0, 0, 'invalid native source is not bound');
function synchronize() {
  observer();
  for (let count = 0; microtasks.length || frames.length; count++) {
    assert.ok(count < 20, 'bounded production synchronization');
    for (const callback of microtasks.splice(0)) callback();
    for (const callback of frames.splice(0)) callback();
  }
}
function click(target) {
  const event = {target, button: 0, defaultPrevented: false, preventDefault() { this.defaultPrevented = true; }};
  if (target.isConnected) emit('click', event);
  for (let node = target; node; node = node.parentElement) for (const callback of node.events.click || []) callback(event);
  if (target.tagName === 'A' && !event.defaultPrevented) location.href = target.href;
  return event;
}
root.setAttribute('data-intelligence-admitted', 'true');
synchronize(); synchronize(); synchronize();
assert.equal(anchor.events.click.length, 1, 'repeated successful validation binds one local guard');
assert.equal(anchor.getAttribute('aria-disabled'), 'false', 'native connected workflow relation remains supported');
assert.equal(click(anchor).defaultPrevented, false, 'connected supported anchor retains its default navigation');
assert.equal(location.hash, '#workflow-studio');
assert.deepEqual([anchor.getAttribute('href'), anchor.dataset.intelligenceRoute, anchor.dataset.intelligenceChannel, root.dataset.intelligenceSource], authored, 'binding preserves authored route and complete native source');
synchronize();
click(root.querySelector('[data-intelligence-dismiss="tenant-workflow-delegation"]'));
assert.equal(anchor.closest('[data-intelligence-card]').hidden, true, 'native local dismissal hides the retained card');
location.hash = '#identity-panel-employees';
assert.equal(click(anchor).defaultPrevented, true, 'document capture refuses the connected hidden route');
assert.equal(location.hash, '#identity-panel-employees');
root.remove();
assert.equal(anchor.isConnected, false);
assert.equal(click(anchor).defaultPrevented, true, 'retained detached hidden route is refused without document propagation');
assert.equal(location.hash, '#identity-panel-employees', 'detached hidden anchor cannot perform native default navigation');
parent.append(fresh); synchronize();
const next = fresh.querySelector('[data-intelligence-route="tenant-workflow-delegation"]');
assert.equal(next.events.click.length, 1, 'replacement receives its own guard');
click(fresh.querySelector('[data-intelligence-reset]'));
assert.equal(click(next).defaultPrevented, false, 'fresh connected supported route still navigates after local reset');
assert.equal(location.hash, '#workflow-studio');
fresh.remove(); location.hash = '#identity-panel-employees';
assert.equal(click(next).defaultPrevented, true, 'retained detached visible route is also refused');
assert.equal(location.hash, '#identity-panel-employees');
console.log('Intelligence native admission, connected navigation and detached route refusal PASS');
"##;

#[test]
fn intelligence_native_admission_refuses_detached_route_defaults() {
    let owner = Owner::new();
    let html = owner.with(|| super::super::document::ServerShell().into_view().to_html());
    owner.cleanup();
    let mut child = Command::new("node")
        .args(["-e", DETACHED_ROUTE_CHECK, super::script().as_str()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node runs the production Intelligence route regression");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(html.as_bytes())
        .expect("Native SSR enters the Intelligence route fixture");
    let output = child
        .wait_with_output()
        .expect("Intelligence route regression completes");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
