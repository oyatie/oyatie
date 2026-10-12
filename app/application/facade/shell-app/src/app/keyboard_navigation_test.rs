use super::ssr_tab_navigation_script;
use std::process::Command;

const CHECK: &str = r#"
const assert = require('node:assert/strict');
const vm = require('node:vm');
const ownerIds = ['settings-center', 'identity-workforce-service', 'finance-commercial-service',
  'cloud-ops-cockpit', 'resource-audit-console'];
const names = ['settings', 'identity', 'finance', 'cockpit', 'resource'];
function fixture({ wired = false } = {}) {
  const events = {}, frames = [];
  function emit(name, target, key, isTrusted = true) {
    const event = { target, key, isTrusted, prevented: false, preventDefault() { this.prevented = true; } };
    if (name === 'click') target.closest('[role="tab"]')?.nativeClick?.(event);
    for (const callback of events[name] || []) callback(event);
    return event;
  }
  class Element {
    constructor(tag = 'DIV', attrs = {}) {
      this.tagName = tag; this.attrs = {...attrs}; this.children = [];
      this.parentElement = null; this.connected = true; this.focusCount = 0;
      this.value = ''; this.clientWidth = 120; this.scrollLeft = 0; this.disabled = false;
      this.classList = {
        contains: name => (this.attrs.class || '').split(/\s+/).includes(name),
        toggle: (name, enabled) => {
          const classes = new Set((this.attrs.class || '').split(/\s+/).filter(Boolean));
          if (enabled) classes.add(name); else classes.delete(name);
          this.attrs.class = [...classes].join(' ');
        },
      };
    }
    get id() { return this.attrs.id || ''; }
    get isConnected() { return this.connected && (!this.parentElement || this.parentElement.isConnected); }
    get dataset() { return Object.fromEntries(Object.entries(this.attrs)
      .filter(([key]) => key.startsWith('data-'))
      .map(([key, value]) => [key.slice(5).replace(/-([a-z])/g, (_, letter) => letter.toUpperCase()), value])); }
    get tabIndex() { return Number(this.attrs.tabindex ?? (this.tagName === 'BUTTON' ? 0 : -1)); }
    set tabIndex(value) { this.attrs.tabindex = String(value); }
    append(child) { child.parentElement = this; this.children.push(child); return child; }
    getAttribute(name) { return this.attrs[name] ?? null; }
    hasAttribute(name) { return Object.hasOwn(this.attrs, name); }
    setAttribute(name, value) { this.attrs[name] = String(value); }
    matches(selector) {
      const id = selector.match(/#([\w-]+)/), cls = selector.match(/\.([\w-]+)/);
      if (id && this.id !== id[1] || cls && !this.classList.contains(cls[1])) return false;
      const tag = selector.match(/^[a-z]+/);
      if (tag && this.tagName !== tag[0].toUpperCase()) return false;
      return [...selector.matchAll(/\[([\w-]+)(?:="([^"]*)")?\]/g)]
        .every(([, name, value]) => value === undefined ? this.hasAttribute(name) : this.getAttribute(name) === value);
    }
    closest(selector) {
      for (let node = this; node; node = node.parentElement) if (node.matches(selector)) return node;
      return null;
    }
    querySelectorAll(selector) {
      return this.children.flatMap(child => [...(child.matches(selector) ? [child] : []),
        ...child.querySelectorAll(selector)]);
    }
    querySelector(selector) { return this.querySelectorAll(selector)[0] || null; }
    focus() { this.focusCount++; document.activeElement = this; emit('focusin', this); }
    click() { return emit('click', this, undefined, false); }
    scrollIntoView() { this.scrolled = true; }
    getClientRects() { return this.isConnected ? [{}] : []; }
  }
  const body = new Element('BODY');
  const shell = body.append(new Element('DIV', {class: 'console-app'}));
  const document = { body, activeElement: body, focused: true, hasFocus() { return this.focused; },
    getElementById: id => id ? body.querySelectorAll(`#${id}`).find(node => node.id === id && node.isConnected) || null : null,
    querySelector: selector => body.querySelectorAll(selector).find(node => node.isConnected) || null,
    querySelectorAll: selector => body.querySelectorAll(selector).filter(node => node.isConnected),
    addEventListener: (name, callback) => (events[name] ||= []).push(callback),
  };
  function attach(index) {
    const name = names[index];
    const owner = shell.append(new Element('SECTION', {id: ownerIds[index]}));
    if (name === 'identity') owner.attrs['data-identity-service'] = 'true';
    const tablist = owner.append(new Element('DIV', {class: `${name}-tabs`, role: 'tablist'}));
    if (wired) tablist.attrs['data-tablist-wired'] = 'true';
    const views = name === 'identity' ? ['auth', 'employees', 'org'] : ['first', 'second', 'third'];
    const tabs = views.map((view, position) => tablist.append(new Element('BUTTON', {
      id: `${name}-tab-${view}`, role: 'tab', 'aria-controls': `${name}-panel-${view}`,
      'aria-selected': String(position === 0), class: position === 0 ? 'active' : '',
      ...(name === 'identity' ? {'data-identity-tab': view} : {}),
    })));
    const panels = views.map((view, position) => owner.append(new Element('ARTICLE', {
      id: `${name}-panel-${view}`, role: 'tabpanel', 'aria-labelledby': tabs[position].id,
      class: position === 0 ? 'active' : '',
      ...(name === 'identity' ? {'data-identity-panel': view} : {}),
    })));
    return {owner, tablist, tabs, panels};
  }
  const groups = names.map((_, index) => attach(index));
  const employeeRoot = groups[1].panels[1].append(new Element('DIV', {
    'data-employee-directory': 'sample', 'data-employee-source': 'exact-source',
  }));
  for (const field of ['search', 'reset', 'count', 'empty', 'scroll'])
    employeeRoot.append(new Element('DIV', {[`data-employee-${field}`]: 'true'}));
  const scroller = employeeRoot.querySelector('[data-employee-scroll]');
  const window = {location: {hash: ''}, addEventListener() {}};
  const context = {document, window, Element, requestAnimationFrame: callback => frames.push(callback),
    MutationObserver: class { observe() {} },
  };
  vm.runInNewContext(process.argv[1], context, {timeout: 1000});
  vm.runInNewContext(process.argv[2], context, {timeout: 1000});
  return {groups, document, emit, attach, scroller, shell,
    nested: tab => tab.append(new Element('SPAN')),
    flush: () => { while (frames.length) frames.shift()(); },
    route: () => groups[1].owner.append(new Element('BUTTON', {'data-identity-tab': 'employees'})),
  };
}
function snapshot(group) {
  return [group.tabs.map(tab => [tab.getAttribute('aria-selected'), tab.tabIndex,
    tab.classList.contains('active'), tab.focusCount]), group.panels.map(panel => panel.classList.contains('active'))];
}
function selected(group, index) {
  assert.deepEqual(group.tabs.map(tab => tab.getAttribute('aria-selected')), group.tabs.map((_, i) => String(i === index)));
  assert.deepEqual(group.panels.map(panel => panel.classList.contains('active')), group.tabs.map((_, i) => i === index));
}
const state = fixture();
for (const group of state.groups) {
  assert.deepEqual(group.tabs.map(tab => tab.tabIndex), [0, -1, -1], 'initial selected tab is the only tab stop');
  state.emit('click', state.nested(group.tabs[1])); selected(group, 1);
  assert.equal(state.document.activeElement, group.tabs[1]);
  assert.equal(group.tabs[1].focusCount, 1, 'direct employee activation has one owner');
  assert.equal(group.tablist.hasAttribute('data-tablist-wired'), false, 'SSR never takes the island marker');
  const move = (key, index) => {
    assert.equal(state.emit('keydown', state.document.activeElement, key).prevented, true);
    assert.equal(state.document.activeElement, group.tabs[index]); selected(group, 1);
    assert.deepEqual(group.tabs.map(tab => tab.tabIndex), group.tabs.map((_, i) => i === index ? 0 : -1));
  };
  move('ArrowRight', 2); move('ArrowDown', 0); move('ArrowLeft', 2); move('ArrowUp', 1);
  move('Home', 0); move('End', 2);
  assert.equal(state.emit('keydown', group.tabs[2], 'Enter').prevented, true); selected(group, 2);
  group.tabs[0].focus();
  assert.equal(state.emit('keydown', group.tabs[0], ' ').prevented, true); selected(group, 0);
  group.tabs[1].focus();
  assert.equal(state.emit('keydown', group.tabs[1], 'Spacebar').prevented, true); selected(group, 1);
  assert.equal(state.emit('keydown', group.tabs[1], 'Tab').prevented, false);
}
const refusals = [
  group => { group.tablist.attrs.class = 'unknown-tabs'; },
  group => { group.tablist.attrs.class += ' hub-tabs'; },
  group => { group.tablist.attrs.class += ' resource-tabs'; },
  group => { group.tablist.attrs['data-native-hub-tabs'] = 'true'; },
  group => { group.tablist.attrs['data-tablist-wired'] = 'true'; },
  group => { group.tabs[1].connected = false; },
  group => { group.owner.connected = false; },
  group => { group.tabs[1].disabled = true; },
  group => { group.tabs[1].attrs['aria-controls'] = 'missing'; },
  group => { group.panels[1].attrs['aria-labelledby'] = 'foreign-tab'; },
  group => { group.panels[1].attrs.role = 'region'; },
  group => { group.panels[1].connected = false; },
  (group, state) => { state.shell.append(group.panels[1]); },
  group => { group.tabs[1].attrs['aria-selected'] = 'true'; },
  (group, state) => { state.nested(group.tabs[0]).append(group.tabs[1]); },
];
for (const mutate of refusals) {
  const state = fixture(), group = state.groups[3]; mutate(group, state);
  group.tabs[1].focus(); const before = snapshot(group);
  state.emit('click', group.tabs[1]);
  assert.equal(state.emit('keydown', group.tabs[1], 'Enter').prevented, false);
  assert.deepEqual(snapshot(group), before, 'malformed/foreign/detached/native ownership refuses all mutation');
}
const hydrated = fixture({wired: true});
for (const group of hydrated.groups) {
  assert.deepEqual(group.tabs.map(tab => tab.tabIndex), [0, 0, 0], 'SSR leaves already wired baseline alone');
  group.tabs[1].nativeClick = event => { if (event.isTrusted) group.tabs[1].focus(); };
  const before = group.tabs.map(tab => tab.getAttribute('aria-selected'));
  hydrated.emit('click', group.tabs[1]);
  assert.equal(group.tabs[1].focusCount, 1, 'hydrated listener exclusively owns activation');
  assert.deepEqual(group.tabs.map(tab => tab.getAttribute('aria-selected')), before);
  assert.equal(hydrated.emit('keydown', group.tabs[1], 'Enter').prevented, false);
  assert.equal(group.tablist.getAttribute('data-tablist-wired'), 'true');
}
const restored = fixture();
for (const [index, group] of restored.groups.entries()) {
  restored.document.activeElement = restored.document.body;
  group.tabs[1].click(); selected(group, 1);
  assert.equal(restored.document.activeElement, restored.document.body,
    'automatic selection leaves detached-control BODY focus available to its restoration owner');
  assert.equal(group.tabs[1].focusCount, 0, 'selection-only restore performs no tab focus call');
  const later = restored.groups[(index + 1) % restored.groups.length].tabs[0];
  restored.emit('click', later);
  const laterFocusCount = later.focusCount;
  group.tabs[2].click(); selected(group, 2);
  assert.equal(restored.document.activeElement, later,
    'automatic selection preserves the later trusted focus in an unrelated tablist');
  assert.equal(later.focusCount, laterFocusCount, 'restoration does not bounce through unrelated focus');
  assert.equal(group.tabs[2].focusCount, 0);
  assert.deepEqual(group.tabs.map(tab => tab.tabIndex), [-1, -1, 0],
    'selection-only restore still establishes its selected roving tab stop');
}
const routing = fixture();
routing.scroller.scrollLeft = 37; routing.emit('scroll', routing.scroller);
routing.route().click(); routing.flush(); selected(routing.groups[1], 1);
assert.equal(routing.document.activeElement, routing.groups[1].tabs[1], 'HR route uses the controlled tab owner');
assert.equal(routing.groups[1].tabs[1].focusCount, 1, 'route activation is not duplicated');
assert.equal(routing.scroller.scrollLeft, 37, 'existing HR route restores remembered directory scroll');
const unavailableRouting = fixture({wired: true});
unavailableRouting.route().click(); unavailableRouting.flush(); selected(unavailableRouting.groups[1], 0);
assert.equal(unavailableRouting.document.activeElement, unavailableRouting.document.body,
  'unavailable native activation leaves the inactive HR destination unfocused');
assert.equal(unavailableRouting.groups[1].tabs[1].focusCount, 0);
const islandRouting = fixture({wired: true}), employees = islandRouting.groups[1];
// This native adapter tests the shipped HR controller; it does not execute WASM.
employees.tabs[1].nativeClick = event => {
  employees.tabs.forEach((tab, index) => {
    const active = index === 1;
    tab.setAttribute('aria-selected', String(active)); tab.tabIndex = active ? 0 : -1;
    tab.classList.toggle('active', active); employees.panels[index].classList.toggle('active', active);
  });
  if (event.isTrusted) employees.tabs[1].focus();
};
islandRouting.route().click(); islandRouting.flush(); selected(employees, 1);
assert.equal(islandRouting.document.activeElement, employees.tabs[1],
  'shipped HR route explicitly owns focus after native selection-only activation');
assert.equal(employees.tabs[1].focusCount, 1, 'hydrated HR route focuses the controlled tab exactly once');
islandRouting.route().click(); islandRouting.flush();
assert.equal(employees.tabs[1].focusCount, 1, 'repeated HR route does not focus an already focused tab twice');
islandRouting.document.activeElement = islandRouting.document.body;
islandRouting.document.focused = false;
islandRouting.route().click(); islandRouting.flush();
assert.equal(islandRouting.document.activeElement, islandRouting.document.body,
  'intentional HR route does not seize focus from an unfocused document');
assert.equal(employees.tabs[1].focusCount, 1);
const stale = routing.groups[3]; stale.owner.connected = false;
const fresh = routing.attach(3); routing.emit('click', fresh.tabs[1]); selected(fresh, 1);
assert.equal(stale.tabs[1].getAttribute('aria-selected'), 'false', 'document lifetime never retains stale controls');
"#;

#[test]
fn ssr_dom_tabs_preserve_owner_manual_activation_and_hydrated_handoff() {
    let script = ssr_tab_navigation_script();
    let hr_script = include_str!("../server/hr_directory.rs")
        .split_once("r#\"")
        .and_then(|(_, source)| source.rsplit_once("\"#;"))
        .map(|(script, _)| script)
        .expect("the served HR directory script remains a raw string constant");
    let output = Command::new("node")
        .args(["-e", CHECK, &script, hr_script])
        .output()
        .expect("node must be available for the SSR DOM tab ownership regression");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
