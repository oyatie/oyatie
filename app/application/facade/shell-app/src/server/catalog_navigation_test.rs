use std::{env, process::Command};

use super::SCRIPT;

const INTERACTION_CHECK: &str = r#"
const assert = require('node:assert/strict');
const vm = require('node:vm');
class FakeElement {
  constructor(tag = 'div', attrs = {}, text = '') {
    this.tag = tag;
    this.attrs = attrs;
    this.textContent = text;
    this.children = [];
    this.isConnected = true;
    this.focusCount = 0;
    this.classes = new Set((attrs.class || '').split(' ').filter(Boolean));
    this.classList = {
      contains: (name) => this.classes.has(name),
      toggle: (name, selected) => selected ? this.classes.add(name) : this.classes.delete(name),
    };
  }
  get id() { return this.attrs.id || ''; }
  get dataset() {
    return Object.fromEntries(Object.entries(this.attrs)
      .filter(([name]) => name.startsWith('data-'))
      .map(([name, value]) => [name.slice(5).replace(/-([a-z])/g, (_, letter) => letter.toUpperCase()), value]));
  }
  append(child) { child.parentElement = this; this.children.push(child); return child; }
  matches(selector) {
    if (!selector.startsWith('[')) return this.tag === selector;
    const [, name, value] = selector.match(/^\[([^=\]]+)(?:="([^"]+)")?\]$/) || [];
    return name in this.attrs && (value === undefined || this.attrs[name] === value);
  }
  closest(selector) {
    for (let current = this; current; current = current.parentElement)
      if (current.matches(selector)) return current;
    return null;
  }
  querySelectorAll(selector) {
    const matches = [];
    for (const child of this.children) {
      if (child.matches(selector)) matches.push(child);
      matches.push(...child.querySelectorAll(selector));
    }
    return matches;
  }
  querySelector(selector) { return this.querySelectorAll(selector)[0] || null; }
  contains(target) {
    for (let current = target; current; current = current.parentElement)
      if (current === this) return true;
    return false;
  }
  setAttribute(name, value) { this.attrs[name] = String(value); }
  scrollIntoView() { this.scrollCount = (this.scrollCount || 0) + 1; }
  focus() {
    this.focusCount += 1;
    document.activeElement = this;
    document.emit('focusin', this);
  }
}
globalThis.Element = FakeElement;
const body = new FakeElement('body');
const shellHost = body.append(new FakeElement('div', { id: 'dashboard-island-root' }));
const outside = body.append(new FakeElement('button'));
const listeners = {};
globalThis.document = {
  body,
  activeElement: body,
  addEventListener: (name, callback) => (listeners[name] ||= []).push(callback),
  emit: (name, target) => (listeners[name] || []).forEach((callback) => callback({ target })),
  querySelector: (selector) => body.querySelector(selector),
  getElementById: (id) => body.querySelector(`[id="${id}"]`),
};
globalThis.window = { location: { hash: '' } };
let notifyMutation;
globalThis.MutationObserver = class {
  constructor(callback) { notifyMutation = callback; }
  observe() {}
};
function ontologyPanel(reason = 'Admin may inspect.', context = 'tenant-admin') {
  const panel = new FakeElement('section', { id: 'ontology-command-console' });
  const graph = panel.append(new FakeElement('div', {
    'data-ontology-console': 'true', 'data-ontology-context': context,
  }));
  const node = graph.append(new FakeElement('button', {
    class: 'selected', 'data-ontology-node': 'Tenant', 'data-sidepeek-desc': 'Tenant owns the graph.',
  }));
  const policy = graph.append(new FakeElement('button', {
    'data-ontology-node': 'Policy', 'data-sidepeek-desc': 'Policy gates the graph.',
  }));
  const lineage = graph.append(new FakeElement('button', { 'data-ontology-action': 'lineage' }));
  const fact = graph.append(new FakeElement('article', { 'data-ontology-fact': 'true' }));
  fact.append(new FakeElement('strong', {}, 'Tenant'));
  fact.append(new FakeElement('em', {}, 'owns modules'));
  fact.append(new FakeElement('p', {}, reason));
  const inspect = fact.append(new FakeElement('button', { 'data-ontology-action': 'inspect-fact' }));
  const workflow = fact.append(new FakeElement('button', { 'data-ontology-action': 'route-workflow' }));
  const detail = graph.append(new FakeElement('strong', { 'data-ontology-detail': 'true' }, 'Default graph status'));
  return { panel, node, policy, lineage, inspect, workflow, detail };
}
function setConnected(element, connected) {
  element.isConnected = connected;
  for (const child of element.children) setConnected(child, connected);
}
function replaceHost(...children) {
  for (const child of shellHost.children) setConnected(child, false);
  if (!document.activeElement.isConnected) document.activeElement = body;
  shellHost.children = [];
  for (const child of children) shellHost.append(child);
  notifyMutation();
}
const card = new FakeElement('article', { 'data-catalog-module': 'true' });
card.append(new FakeElement('button', { 'data-sidepeek-id': 'CAT-OYATIE-ONTOLOGY' }));
const open = card.append(new FakeElement('button', { 'data-catalog-action': 'open' }));
const oldGraph = ontologyPanel();
shellHost.append(card);
shellHost.append(oldGraph.panel);
vm.runInThisContext(process.argv[1]);

document.emit('click', open);
assert.equal(document.activeElement, oldGraph.panel);
assert.equal(window.location.hash, '#ontology-command-console');
document.emit('click', oldGraph.inspect);
assert.match(oldGraph.detail.textContent, /Tenant · owns modules/);
document.activeElement = body;
replaceHost();
assert.equal(document.activeElement, body, 'focus waits for the async island envelope');
const workflowPanel = new FakeElement('section', { id: 'workflow-studio' });
const newGraph = ontologyPanel();
replaceHost(workflowPanel, newGraph.panel);
assert.equal(document.activeElement, newGraph.panel, 'focus follows the replacement panel once');
assert.equal(newGraph.panel.focusCount, 1);
assert.equal(newGraph.detail.textContent, 'Tenant · owns modules · Admin may inspect.');
assert.equal(newGraph.detail.attrs.role, 'status');
assert.equal(newGraph.node.attrs['aria-pressed'], 'true');
notifyMutation();
assert.equal(newGraph.panel.focusCount, 1);

newGraph.policy.focus();
document.emit('click', newGraph.policy);
newGraph.lineage.focus();
document.emit('click', newGraph.lineage);
assert.equal(newGraph.detail.textContent, 'Current permitted relations: Tenant · owns modules');
const lineageGraph = ontologyPanel();
replaceHost(workflowPanel, lineageGraph.panel);
assert.equal(lineageGraph.detail.textContent, 'Current permitted relations: Tenant · owns modules');
assert.equal(document.activeElement, lineageGraph.lineage, 'matching lineage control keeps focus');
assert.equal(lineageGraph.policy.attrs['aria-pressed'], 'true', 'lineage preserves selected node');

lineageGraph.policy.focus();
document.emit('click', lineageGraph.policy);
const selectedGraph = ontologyPanel();
replaceHost(workflowPanel, selectedGraph.panel);
assert.equal(selectedGraph.policy.attrs['aria-pressed'], 'true', 'node selection survives replacement');
assert.equal(selectedGraph.node.attrs['aria-pressed'], 'false');
assert.equal(selectedGraph.detail.textContent, 'Policy selected · Policy gates the graph.');
assert.equal(document.activeElement, selectedGraph.policy, 'matching selected node keeps focus');

selectedGraph.inspect.focus();
document.emit('click', selectedGraph.inspect);
const matchingFactGraph = ontologyPanel();
replaceHost(workflowPanel, matchingFactGraph.panel);
assert.equal(document.activeElement, matchingFactGraph.inspect, 'matching fact control keeps focus');
assert.equal(matchingFactGraph.detail.textContent, 'Tenant · owns modules · Admin may inspect.');
assert.equal(matchingFactGraph.policy.attrs['aria-pressed'], 'true', 'fact detail preserves selected node');
document.emit('click', matchingFactGraph.inspect);
const changedGraph = ontologyPanel('Clinician may inspect.');
replaceHost(workflowPanel, changedGraph.panel);
assert.equal(changedGraph.detail.textContent, 'Default graph status', 'different facts reset state');
assert.equal(changedGraph.node.attrs['aria-pressed'], 'true');
assert.equal(document.activeElement, body, 'different facts drop stale focus');

document.emit('click', changedGraph.workflow);
assert.equal(document.activeElement, workflowPanel);
assert.equal(window.location.hash, '#workflow-studio');

window.location.hash = '#ontology-command-console';
changedGraph.policy.focus();
document.emit('click', changedGraph.policy);
changedGraph.inspect.focus();
document.emit('click', changedGraph.inspect);
const differentContext = ontologyPanel('Clinician may inspect.', 'corporate-office');
replaceHost(differentContext.panel);
assert.equal(differentContext.detail.textContent, 'Default graph status', 'same facts in another context reset detail');
assert.equal(differentContext.node.attrs['aria-pressed'], 'true', 'another context resets selection');
assert.equal(document.activeElement, body, 'another context drops old focus');

const laterCard = new FakeElement('article', { 'data-catalog-module': 'true' });
laterCard.append(new FakeElement('button', { 'data-sidepeek-id': 'CAT-OYATIE-ONTOLOGY' }));
const laterOpen = laterCard.append(new FakeElement('button', { 'data-catalog-action': 'open' }));
shellHost.append(laterCard);
document.emit('click', laterOpen);
outside.focus();
const skippedGraph = ontologyPanel();
replaceHost(skippedGraph.panel);
assert.equal(document.activeElement, outside, 'intentional focus elsewhere is preserved');
assert.equal(skippedGraph.panel.focusCount, 0);
"#;

#[test]
#[ignore = "manual browser-script check; set NODE_BINARY to a Node executable"]
fn ontology_navigation_keeps_focus_across_island_replacement() {
    let output =
        Command::new(env::var("NODE_BINARY").expect("set NODE_BINARY to run this manual check"))
            .args(["-e", INTERACTION_CHECK, SCRIPT])
            .output()
            .expect("NODE_BINARY must run the served browser script");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
