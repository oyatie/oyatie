use std::process::Command;

const REPORT_CHECK: &str = r##"
const assert = require('node:assert/strict'), vm = require('node:vm'), crypto = require('node:crypto');
const source = process.argv[1];
function block(marker, end) {
  const start = source.indexOf(marker), finish = source.indexOf(end, start);
  assert.ok(start >= 0 && finish > start, 'production block exists: ' + marker);
  return source.slice(start, finish + end.length);
}
const start = source.indexOf('  function report'), end = source.indexOf('  function openChannel(');
assert.ok(start >= 0 && end > start, 'production report functions exist');
const production = source.slice(start, end) + block('  function route(anchor, event)', '\n  }\n')
  + block("  document.addEventListener('focusin'", '\n  });')
  + block("  document.addEventListener('click'", '\n  }, true);')
  + block("  window.addEventListener('hashchange'", '\n  });');
const opening = channel => `Opening ${channel} existing sample; no output created.`;
const success = channel => `${channel} existing sample opened; no draft, message or receipt created.`;
function setup(hash = '#before', ready = true) {
  const status = {textContent:''}, clock = {now:0}, events = {}, raf = [], timers = [];
  class Element { constructor(id) { this.id = id; this.isConnected = true; } closest() { return null; } }
  const run = {status, clock, events, raf, timers, ready, nativeCalls:0, routeCalls:0, destination:new Element('work-hub')};
  const root = {contains: () => false, querySelector: () => status};
  const context = vm.createContext({Element, location:{hash, href:'https://preview.invalid/' + hash},
    document:{body:new Element('body'), getElementById: id => id === 'work-hub' ? run.destination : null,
      addEventListener: (name, callback) => events[name] = callback},
    window:{addEventListener: (name, callback) => events[name] = callback},
    host:{contains: target => target === run.destination}, all: () => [run.destination],
    performance:{now: () => clock.now}, root,
    destinationFor: () => run.destination, routeIdentity: anchor => ['work-hub', anchor.channel],
    openChannel: () => { run.nativeCalls++; return true; }, revealSSR: () => { throw Error('island route'); },
    workspace: () => ({}), channelReady: () => run.ready,
    text: (node, value) => node.textContent = value, cancelFocus: () => {},
    requestAnimationFrame: callback => raf.push(callback), setTimeout: (callback, delay) => {
      assert.equal(delay, 5000, 'original report expiry'); timers.push(callback);
    }, fetch: () => { throw Error('no network effect'); }});
  vm.runInContext("let current = {root, key:'owner', mode:'island', available:true, source:['context','unused','hub']}; let report = null, handoff = null, dispatching = false;\n" + production, context);
  run.call = expression => vm.runInContext(expression, context);
  run.route = (channel = 'Mail') => { run.routeCalls++; run.call(`route({channel:${JSON.stringify(channel)}}, {defaultPrevented:false, preventDefault(){throw Error('unexpected refusal')}})`); };
  run.settle = () => run.call('settleReport()');
  run.hash = value => { context.location.hash = value; context.location.href = 'https://preview.invalid/' + value; events.hashchange(); };
  run.arrive = () => { run.hash('#work-hub'); raf.splice(0).forEach(callback => callback()); };
  run.focus = target => events.focusin({target});
  run.foreign = id => new Element(id);
  run.pending = () => run.call('report !== null');
  return run;
}
const results = [];
function check(id, action) {
  const run = setup();
  try { action(run); assert.equal(run.nativeCalls, run.routeCalls, 'only original route channel handoffs');
    assert.equal(run.timers.length, run.routeCalls, 'one original expiry per route');
    results.push({id, result:'PASS'});
  } catch (error) { results.push({id, result:'FAIL', error:error.message}); }
}
check('pending-prehash-settlement', run => {
  run.route(); run.settle(); assert.equal(run.pending(), true, 'own report remains pending before hash');
  run.arrive(); assert.equal(run.status.textContent, success('Mail'));
});
check('own-destination-focus-before-hash', run => {
  run.route(); run.focus(run.destination); assert.equal(run.pending(), true, 'own prehash focus retains pending report');
  run.arrive(); assert.equal(run.status.textContent, success('Mail'));
});
check('matching-hash-ready', run => { run.hash('#work-hub'); run.route(); run.arrive(); assert.equal(run.status.textContent, success('Mail')); });
check('matching-hash-not-ready', run => {
  run.ready = false; run.hash('#work-hub'); run.route(); run.arrive(); assert.equal(run.status.textContent, opening('Mail'));
  assert.equal(run.pending(), true); run.ready = true; run.settle(); assert.equal(run.status.textContent, success('Mail'));
});
for (const [id, foreignId] of [['unrelated-focus','other'], ['foreign-same-id-focus','work-hub']]) check(id, run => {
  run.ready = false; run.hash('#work-hub'); run.route(); run.focus(run.foreign(foreignId));
  run.ready = true; run.settle(); assert.equal(run.pending(), false); assert.notEqual(run.status.textContent, success('Mail'));
});
check('unrelated-click', run => {
  run.route(); run.events.click({target:run.foreign('other')}); run.arrive(); assert.equal(run.pending(), false); assert.notEqual(run.status.textContent, success('Mail'));
});
check('unrelated-hash', run => {
  run.ready = false; run.route(); run.hash('#other'); run.ready = true; run.arrive(); assert.equal(run.pending(), false); assert.notEqual(run.status.textContent, success('Mail'));
});
check('newer-route-refuses-old-callbacks', run => {
  run.ready = false; run.hash('#work-hub'); run.route(); const oldFrame = run.raf.shift(), oldTimer = run.timers[0];
  run.clock.now = 1000; run.route('Community'); run.clock.now = 5000; oldFrame(); oldTimer();
  assert.equal(run.status.textContent, opening('Community')); run.ready = true; run.arrive(); assert.equal(run.status.textContent, success('Community'));
});
for (const [id, change] of [['unavailable-owner','current.available = false'], ['changed-key',"current.key = 'other'"],
  ['changed-mode',"current.mode = 'ssr'"], ['changed-source',"current.source = ['context','unused','other']"]]) check(id, run => {
  run.ready = false; run.hash('#work-hub'); run.route(); run.call(change); run.ready = true; run.settle();
  assert.equal(run.pending(), false); assert.notEqual(run.status.textContent, success('Mail')); run.timers[0](); assert.notEqual(run.status.textContent, success('Mail'));
});
check('same-semantic-owner-handoff', run => {
  run.hash('#work-hub'); run.route(); run.destination = run.foreign('work-hub'); run.call('current.root = {...root}');
  run.focus(run.destination); run.arrive(); assert.equal(run.status.textContent, success('Mail'));
});
check('original-expiry-refuses-late-completion', run => {
  run.ready = false; run.hash('#work-hub'); run.route(); run.clock.now = 5000; run.timers[0]();
  assert.equal(run.status.textContent, 'Mail sample outcome unavailable; no output created.');
  run.ready = true; run.arrive(); run.settle(); assert.equal(run.pending(), false); assert.notEqual(run.status.textContent, success('Mail'));
});
check('pending-prehash-original-expiry', run => {
  run.route(); run.clock.now = 5000; run.timers[0]();
  assert.equal(run.status.textContent, 'Mail sample outcome unavailable; no output created.');
  run.arrive(); assert.equal(run.pending(), false); assert.notEqual(run.status.textContent, success('Mail'));
});
check('expired-before-delayed-timer-refuses-success', run => {
  run.ready = false; run.hash('#work-hub'); run.route(); run.clock.now = 5001; run.ready = true; run.settle();
  assert.equal(run.pending(), false); assert.notEqual(run.status.textContent, success('Mail')); run.timers[0](); assert.notEqual(run.status.textContent, success('Mail'));
});
console.log(JSON.stringify({schema:'oyatie.catalog.native-report-regression/1', productionScriptSha256:crypto.createHash('sha256').update(source).digest('hex'), results}));
assert.deepEqual(results.filter(row => row.result === 'FAIL'), [], 'all production report lifetime and cancellation cases');
"##;

#[test]
fn catalog_route_reports_preserve_pending_arrival_and_cancel_stale_completion() {
    let output = Command::new("node")
        .args(["-e", REPORT_CHECK, super::super::script().as_str()])
        .output()
        .expect("Node runs the production Catalog report regression");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
