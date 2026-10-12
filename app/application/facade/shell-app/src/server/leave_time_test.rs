use leptos::prelude::*;
use std::io::Write;
use std::process::{Command, Stdio};

const CHECK: &str = r###"
const assert = require('node:assert/strict'), vm = require('node:vm');
const input = JSON.parse(require('node:fs').readFileSync(0,'utf8'));
const events = {}, windowEvents = {}, microtasks = [], frames = [];
const observers = []; let focused = true;
const decode = value => value.replace(/&(quot|apos|amp|lt|gt|#x[0-9a-f]+|#[0-9]+);/gi, (_, name) => name[0] === '#'
  ? String.fromCodePoint(name[1].toLowerCase() === 'x' ? parseInt(name.slice(2),16) : Number(name.slice(1)))
  : ({quot:'"',apos:"'",amp:'&',lt:'<',gt:'>'})[name.toLowerCase()]);
const escape = value => String(value).replaceAll('&','&amp;').replaceAll('"','&quot;').replaceAll('<','&lt;').replaceAll('>','&gt;');
const voids = new Set(['input','meta','link','br','hr','img']);
class Comment {
  constructor(data) { this.data=data; this.nodeType=8; }
  get textContent() { return ''; }
  get outerHTML() { return `<!--${this.data}-->`; }
  cloneNode() { return new Comment(this.data); }
  remove() { Element.prototype.remove.call(this); }
}
class Element {
  constructor(tag,attrs={}) { this.tagName=tag.toUpperCase(); this.attrs=attrs; this.parts=[]; this.events={}; }
  get children() { return this.parts.filter(part => part instanceof Element); }
  get firstElementChild() { return this.children[0] || null; }
  get cells() { return this.children.filter(node => ['TD','TH'].includes(node.tagName)); }
  get nextElementSibling() { const nodes=this.parentElement?.children || [];return nodes[nodes.indexOf(this)+1] || null; }
  get attributes() { return Object.entries(this.attrs).map(([name,value])=>({name,value})); }
  get value() { return this.localValue ?? (this.tagName === 'SELECT' ? this.querySelector('option')?.getAttribute('value') || '' : this.getAttribute('value') || ''); }
  set value(value) { this.localValue=String(value); }
  get defaultValue() { return this.getAttribute('value') || ''; }
  get options() { return this.querySelectorAll('option'); }
  get defaultSelected() { return this.hasAttribute('selected'); }
  get selectedOptions() { return this.querySelectorAll('option').filter(node=>node.getAttribute('value')===this.value); }
  get id() { return this.attrs.id || ''; }
  get dataset() { return Object.fromEntries(Object.entries(this.attrs).filter(([name])=>name.startsWith('data-')).map(([name,value])=>[name.slice(5).replace(/-([a-z])/g,(_,letter)=>letter.toUpperCase()),value])); }
  get isConnected() { return this===tree || !!this.parentElement?.isConnected; }
  get textContent() { return this.parts.map(part=>typeof part==='string'?part:part.textContent).join(''); }
  set textContent(value) { this.children.forEach(node=>node.parentElement=null); this.parts=[String(value)]; }
  get innerHTML() { return this.parts.map(part=>typeof part==='string'?escape(part):part.outerHTML).join(''); }
  set innerHTML(value) { this.children.forEach(node=>node.parentElement=null); this.parts=[]; parse(value,this); }
  get outerHTML() { return `<${this.tagName.toLowerCase()}${Object.entries(this.attrs).map(([key,value])=>` ${key}="${escape(value)}"`).join('')}>${voids.has(this.tagName.toLowerCase())?'':this.innerHTML+`</${this.tagName.toLowerCase()}>`}`; }
  get disabled() { return this.hasAttribute('disabled'); }
  set disabled(value) { value?this.setAttribute('disabled',''):this.removeAttribute('disabled'); }
  get hidden() { return this.hasAttribute('hidden'); }
  set hidden(value) { value?this.setAttribute('hidden',''):this.removeAttribute('hidden'); }
  get open() { return this.hasAttribute('open'); }
  set open(value) { value?this.setAttribute('open',''):this.removeAttribute('open'); }
  get style() { return {getPropertyValue:name=>this.styles?.[name] || '',setProperty:(name,value)=>{(this.styles ||= {})[name]=value;this.attrs.style=Object.entries(this.styles).map(([key,value])=>`${key}: ${value}`).join(';');}}; }
  get classList() { const list=()=>new Set((this.attrs.class||'').split(/\s+/).filter(Boolean));return {contains:name=>list().has(name),toggle:(name,value)=>{const names=list();if(value)names.add(name);else names.delete(name);this.attrs.class=[...names].join(' ');}}; }
  getAttribute(name) { return this.attrs[name]??null; }
  hasAttribute(name) { return Object.hasOwn(this.attrs,name); }
  setAttribute(name,value) { this.attrs[name]=String(value); }
  removeAttribute(name) { delete this.attrs[name]; }
  append(node) { node.remove();node.parentElement=this;this.parts.push(node);return node; }
  remove() { if(this.parentElement){this.parentElement.parts=this.parentElement.parts.filter(part=>part!==this);this.parentElement=null;} }
  replaceWith(node) { const parent=this.parentElement,index=parent.parts.indexOf(this);this.parentElement=null;node.parentElement=parent;parent.parts[index]=node; }
  before(node) { node.remove();node.parentElement=this.parentElement;this.parentElement.parts.splice(this.parentElement.parts.indexOf(this),0,node); }
  contains(node) { for(;node;node=node.parentElement)if(node===this)return true;return false; }
  matches(selector) { return selector.split(',').some(part=>{const terms=part.trim().split(/\s+(?![^[]*\])/);let node=this;const match=(node,term)=>{
let parsed=selectorTerms.get(term);
if(!parsed){const attrs=[...term.matchAll(/\[([\w-]+)(?:="([^"]*)")?\]/g)],plain=term.replace(/\[[^\]]*\]/g,'');parsed={attrs,tag:plain.match(/^[a-z][\w-]*/i)?.[0].toUpperCase(),ids:[...plain.matchAll(/#([\w-]+)/g)].map(([,id])=>id),classes:[...plain.matchAll(/\.([\w-]+)/g)].map(([,name])=>name)};selectorTerms.set(term,parsed);}
const {attrs,tag,ids,classes}=parsed;
return (!tag||node.tagName===tag)&&ids.every(id=>node.id===id)
  &&classes.every(name=>node.classList.contains(name))
  &&attrs.every(([,name,value])=>value===undefined?node.hasAttribute(name):node.getAttribute(name)===value);
  };if(!match(node,terms.pop()))return false;while(terms.length){const term=terms.pop();do{node=node.parentElement;}while(node&&!match(node,term));if(!node)return false;}return true;}); }
  closest(selector) { for(let node=this;node;node=node.parentElement)if(node.matches(selector))return node;return null; }
  querySelectorAll(selector) { if(selector==='#'||selector.startsWith('#['))throw new SyntaxError('Invalid CSS selector: '+selector);selectors.push(selector);const nodes=[];const visit=node=>{for(const child of node.children){if(child.matches(selector))nodes.push(child);visit(child);}};visit(this);return nodes; }
  querySelector(selector) {
    if(selector==='#'||selector.startsWith('#['))throw new SyntaxError('Invalid CSS selector: '+selector);
    selectors.push(selector);
    const visit=node=>{for(const child of node.children){if(child.matches(selector))return child;const found=visit(child);if(found)return found;}return null;};
    return visit(this);
  }
  cloneNode() { const copy=new Element(this.tagName,{...this.attrs});if(this.localValue!==undefined)copy.localValue=this.localValue;for(const part of this.parts)typeof part==='string'?copy.parts.push(part):copy.append(part.cloneNode(true));if(this.content)copy.content=this.content.cloneNode(true);return copy; }
  getClientRects() { const panel=this.closest('.identity-panel, .finance-panel, .cockpit-panel, .resource-panel');return this.isConnected&&!this.closest('[hidden]')&&!(panel&&!panel.classList.contains('active'))?[{}]:[]; }
  focus() { document.activeElement=this;emit('focusin',{target:this}); }
  scrollIntoView() { this.scrolled=true; }
  close() { this.open=false; }
  click() { click(this); }
  addEventListener(name,callback) { (this.events[name] ||= []).push(callback); }
}
function parse(html,root) {
  const stack=[root];
  html=html.replace(/<(script|style)\b[^>]*>[\s\S]*?<\/\1>/gi,'');
  for(const [token] of html.matchAll(/<!--[\s\S]*?-->|<![^>]*>|<\/[a-z][^>]*>|<[a-z][^>]*>|[^<]+/gi)) {
if(token.startsWith('<!--')) { stack.at(-1).append(new Comment(token.slice(4,-3)));continue; }
if(token.startsWith('<!'))continue;
if(token.startsWith('</')) { const tag=token.match(/^<\/([\w-]+)/)[1].toUpperCase(); if(tag==='TEMPLATE')stack.pop();else assert.equal(stack.pop().tagName,tag);continue; }
if(!token.startsWith('<')) { stack.at(-1).parts.push(decode(token));continue; }
const tag=token.match(/^<([\w-]+)/)[1],attrs={};
for(const [,name,double,single,bare] of token.slice(tag.length+1,-1).matchAll(/([^\s=/>]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+)))?/g))attrs[name]=decode(double??single??bare??'');
const node=stack.at(-1).append(new Element(tag,attrs));
if(tag==='template'){node.content=new Element('fragment');stack.push(node.content);}
else if(!voids.has(tag)&&!token.endsWith('/>'))stack.push(node);
  }
  assert.equal(stack.length,1);
}
const selectors=[],selectorTerms=new Map(),tree=new Element('body');parse(input.html,tree);
const NodeFilter={SHOW_COMMENT:128};
function firstLiteralId(node,id) {
  for(const child of node.parts) {
    if(!(child instanceof Element))continue;
    if(child.hasAttribute('id')&&child.id===id)return child;
    const found=firstLiteralId(child,id);if(found)return found;
  }
  return null;
}
const document={body:tree,activeElement:tree,hasFocus:()=>focused,getElementById:id=>id===''?null:firstLiteralId(tree,id),querySelectorAll:selector=>tree.querySelectorAll(selector),querySelector:selector=>tree.querySelector(selector),createElement:tag=>new Element(tag),createComment:data=>new Comment(data),createTreeWalker:(root,whatToShow)=>{
  assert.equal(whatToShow,NodeFilter.SHOW_COMMENT);
  const comments=[],visit=node=>{for(const child of node.parts||[]){if(child instanceof Comment)comments.push(child);else if(child instanceof Element)visit(child);}};visit(root);
  let index=0;return {nextNode:()=>comments[index++]||null};
},addEventListener:(name,callback,capture=false)=>(events[name] ||= []).push({callback,capture})};
const idProbe=tree.append(new Element('div')), literalId='fixture.[literal]#id';
assert.equal(document.getElementById(''),null,'missing IDs cannot match an empty lookup');
idProbe.append(new Element('span',{id:''}));assert.equal(document.getElementById(''),null,'empty ID attributes do not name a destination');
const firstId=idProbe.append(new Element('section')).append(new Element('span',{id:literalId})), secondId=idProbe.append(new Element('span',{id:literalId}));
assert.equal(document.getElementById(literalId),firstId,'first literal ID follows physical tree order');
const replacedId=new Element('span',{id:literalId});firstId.replaceWith(replacedId);
assert.equal(document.getElementById(literalId),replacedId,'lookup sees physical replacement');
replacedId.remove();assert.equal(document.getElementById(literalId),secondId,'removal exposes next duplicate');
secondId.setAttribute('id','fixture-renamed-id');assert.equal(document.getElementById(literalId),null);assert.equal(document.getElementById('fixture-renamed-id'),secondId);
idProbe.remove();assert.equal(document.getElementById('fixture-renamed-id'),null,'detached IDs are absent');

const queryProbe=tree.append(new Element('div',{class:'fixture-first-match'})), queryFirst=queryProbe.append(new Element('section')).append(new Element('span',{class:'fixture-first-match'})), querySecond=queryProbe.append(new Element('span',{class:'fixture-first-match fixture-last-match'}));
const querySelector='.fixture-last-match, .fixture-first-match';
assert.equal(queryProbe.querySelector(querySelector),queryFirst,'first descendant follows tree order, not selector order');
assert.deepEqual(queryProbe.querySelectorAll(querySelector),[queryFirst,querySecond],'all-match lookup retains duplicates');
querySecond.matches=()=>assert.fail('first-match lookup must stop');assert.equal(queryProbe.querySelector(querySelector),queryFirst);delete querySecond.matches;
queryFirst.before(querySecond);assert.equal(queryProbe.querySelector(querySelector),querySecond,'lookup sees physical reordering');
const queryReplacement=new Element('span',{class:'fixture-first-match'});querySecond.replaceWith(queryReplacement);assert.equal(queryProbe.querySelector(querySelector),queryReplacement,'lookup sees physical replacement');
queryReplacement.remove();assert.equal(queryProbe.querySelector(querySelector),queryFirst,'removal exposes next match');
queryFirst.setAttribute('class','fixture-renamed-match');assert.equal(queryProbe.querySelector(querySelector),null);assert.equal(queryProbe.querySelector('.fixture-renamed-match'),queryFirst);
queryFirst.setAttribute('data-fixture-match','first');assert.equal(queryProbe.querySelector('section span[data-fixture-match="first"]'),queryFirst);
queryFirst.setAttribute('data-fixture-match','second');assert.equal(queryProbe.querySelector('section span[data-fixture-match="first"]'),null);assert.equal(queryProbe.querySelector('section span[data-fixture-match="second"]'),queryFirst);
queryFirst.setAttribute('id','fixture-cached-id');assert.equal(queryProbe.querySelector('#fixture-cached-id'),queryFirst);queryFirst.setAttribute('id','fixture-renamed-id');assert.equal(queryProbe.querySelector('#fixture-cached-id'),null);
queryFirst.parentElement.setAttribute('class','fixture-cached-parent');assert.equal(queryProbe.querySelector('.fixture-cached-parent [data-fixture-match]'),queryFirst);queryFirst.parentElement.removeAttribute('class');assert.equal(queryProbe.querySelector('.fixture-cached-parent [data-fixture-match]'),null);
queryProbe.remove();assert.equal(document.querySelector('.fixture-renamed-match'),null,'document excludes detached matches');assert.equal(queryProbe.querySelector('.fixture-renamed-match'),queryFirst,'detached subtree remains queryable');
for(const selector of ['#','#[invalid]'])for(const root of [tree,new Element('div')]) {assert.throws(()=>root.querySelector(selector),SyntaxError);assert.throws(()=>root.querySelectorAll(selector),SyntaxError);}

const location=new URL('https://preview.invalid/');
if(input.scenario==='fragments')location.hash='#leave-sample-request';
const emit=(name,event)=>{for(const capture of [true,false])for(const listener of events[name]||[])if(!!listener.capture===capture&&!event.stopped)listener.callback(event);};
function click(target,options={}) { const event={target,button:0,isTrusted:false,defaultPrevented:false,...options,preventDefault(){this.defaultPrevented=true;},stopPropagation(){this.stopped=true;}};if(target.isConnected)for(const listener of events.click||[])if(listener.capture&&!event.stopped)listener.callback(event);if(!event.stopped)for(let node=target;node;node=node.parentElement)for(const callback of node.events.click||[])callback(event);if(target.isConnected)for(const listener of events.click||[])if(!listener.capture&&!event.stopped)listener.callback(event);if(target.tagName==='A'&&!event.defaultPrevented)location.hash=target.getAttribute('href');if(target.tagName==='SUMMARY'&&!event.defaultPrevented&&target.parentElement?.tagName==='DETAILS'){const detail=target.parentElement;detail.open=!detail.open;emit('toggle',{target:detail});}return event; }
class HTMLDetailsElement { static [Symbol.hasInstance](node) { return node.tagName==='DETAILS'; } }

const context={document,location,Element,HTMLDetailsElement,NodeFilter,URL,innerWidth:1280,innerHeight:800,window:{addEventListener:(name,callback)=>(windowEvents[name] ||= []).push(callback)},queueMicrotask:callback=>microtasks.push(callback),requestAnimationFrame:callback=>frames.push(callback),MutationObserver:class{constructor(callback){observers.push(callback);}observe(){}}};
vm.runInNewContext(input.policy,context,{timeout:1000});
vm.runInNewContext(input.tabs,context,{timeout:1000});
vm.runInNewContext(input.finance,context,{timeout:1000});
vm.runInNewContext(input.bridge,context,{timeout:1000});
const find=selector=>document.querySelector(selector), root=()=>find('#finance-preview'), panel=()=>root().querySelector('#finance-panel-leave, #leave-time');
const flush=()=>{observers.forEach(callback=>callback());for(let count=0;microtasks.length||frames.length;count++){assert.ok(count<20,'bounded source synchronization');for(const callback of microtasks.splice(0))callback();for(const callback of frames.splice(0))callback();}};
function admitted(ready=true) { for(const node of panel().querySelectorAll('#leave-sample-search, #leave-sample-backup, [data-finance-action], [data-finance-anchor-action], [data-leave-close]'))assert.equal(node.disabled,!ready);assert.equal(find('#finance-tab-leave').disabled,!ready); }
function ledger() { click(find('#finance-tab-ledger'));location.hash='';flush();assert.equal(find('#finance-tab-ledger').getAttribute('aria-selected'),'true'); }
if(!input.scenario||input.scenario==='current') {
admitted();
// Each current SSR entry has its own effect from an empty hash and Ledger selection.
for(const selector of ['[data-finance-side="leave"]','#finance-tab-leave','a.rail-nav[href="#finance-panel-leave"]']) {
  ledger();const control=find(selector);assert.ok(control,selector);click(control);flush();assert.equal(find('#finance-tab-leave').getAttribute('aria-selected'),'true',selector);assert.equal(panel().classList.contains('active'),true);assert.ok(document.activeElement===panel()||document.activeElement===find('#finance-tab-leave'),selector+' meaningful focus');
}
for(const route of ['workflow','cloud','policy','audit','ledger','evidence','mail','community']) {
  click(find('#finance-tab-leave'));click(find(`[data-leave-source] [data-finance-anchor-action="route-${route}"]`));flush();assert.match(find('[data-leave-status]').textContent,/sample preview opened locally/,'existing '+route+' route');
}
click(find('#finance-tab-leave'));
click(find('[data-finance-action="approve-leave"]'));flush();assert.equal(find('[data-leave-detail="request"]').open,true);assert.equal(document.activeElement.id,'leave-sample-request-summary');
click(find('[data-finance-action="reassign-time"]'));flush();assert.equal(document.activeElement.id,'leave-sample-backup');const select=find('#leave-sample-backup');select.value='seojun';emit('change',{target:select});assert.match(find('#leave-coverage-preview').textContent,/박서준/);
click(find('[data-leave-close="coverage"]'));assert.equal(find('[data-leave-detail="coverage"]').open,false);
const search=find('#leave-sample-search');search.value='cutoff';search.focus();emit('input',{target:search});assert.match(find('#leave-sample-count').textContent,/1 sample item/);
for(let round=0;round<2;round++) { const old=root(),replacement=old.cloneNode(true);old.replaceWith(replacement);document.activeElement=tree;flush();admitted();assert.equal(find('#leave-sample-search').value,'cutoff');assert.equal(find('#leave-sample-backup').value,'seojun');assert.equal(find('[data-leave-detail="coverage"]').open,false);assert.equal(document.activeElement,find('#leave-sample-search'));assert.equal(click(old.querySelector('[data-finance-side="leave"]')).defaultPrevented,true,'detached authored default refused'); }
// Current source refusal precedes modifier filtering; healthy native modified defaults remain admitted.
const anchor=find('[data-finance-side="leave"]');assert.equal(click(anchor,{ctrlKey:true}).defaultPrevented,false);location.hash='';
for(const attribute of ['data-finance-source-available','data-finance-available']) {const original=root().getAttribute(attribute);root().setAttribute(attribute,'false');flush();admitted(false);assert.equal(click(anchor,{ctrlKey:true}).defaultPrevented,true);root().setAttribute(attribute,original);flush();admitted();}
const host=find('#dashboard-island-root'), parent=root().parentElement;
for(const kind of ['duplicate-root','duplicate-host','invalid-never-bound']) {
  const original=root();let addition;
  if(kind==='duplicate-root')addition=parent.append(original.cloneNode(true));
  else if(kind==='duplicate-host')addition=tree.append(new Element('div',{id:host.id}));
  else {addition=original.cloneNode(true);addition.setAttribute('data-finance-source','invalid');original.replaceWith(addition);}
  flush();const owned=kind==='invalid-never-bound'?addition.querySelector('[data-finance-side="leave"]'):anchor;assert.equal(click(owned,{metaKey:true}).defaultPrevented,true,kind+' refuses authored default');
  if(kind==='invalid-never-bound')addition.replaceWith(original);else addition.remove();flush();admitted();
}
// A visible local edit is not a new source; literal sample tampering is rejected.
const title=find('#leave-panel-title'), titleText=title.textContent;title.textContent='tampered';flush();admitted(false);
for(const side of ['vendors','billing'])assert.equal(click(find(`[data-finance-side="${side}"]`),{ctrlKey:true}).defaultPrevented,false,'neighbor '+side+' default preserved');
title.textContent=titleText;location.hash='';flush();admitted();
const iam=find('#identity-tenant')||find('[data-identity-tenant]')||tree.append(new Element('input',{id:'independent-iam-fixture'}));iam.value='real-tenant';iam.focus();const old=root(),replacement=old.cloneNode(true);old.replaceWith(replacement);flush();assert.equal(document.activeElement,iam,'latest outside IAM focus wins');assert.equal(iam.value,'real-tenant');
// The own hash event settles without replaying over a later native choice.
// Flush each separate task boundary so Finance observes the latest canonical selection.
ledger();const entryURL=location.href;click(find('[data-finance-side="leave"]'));const leaveURL=location.href;click(find('#finance-tab-ledger'));flush();for(const callback of windowEvents.hashchange||[])callback({oldURL:entryURL,newURL:leaveURL});assert.equal(find('#finance-tab-ledger').getAttribute('aria-selected'),'true');flush();
// Latest native tab choice survives same-source replacement despite stale Leave URL.
location.hash='#finance-panel-leave';const changed=root().cloneNode(true);root().replaceWith(changed);document.activeElement=tree;flush();assert.equal(find('#finance-tab-ledger').getAttribute('aria-selected'),'true');
// An own event cannot settle a different current source or authorize local controls.
ledger();const mismatchOld=location.href;click(find('[data-finance-side="leave"]'));const mismatchURL=location.href;click(find('#finance-tab-ledger'));const contextID=panel().dataset.leaveContext;panel().setAttribute('data-leave-context','corporate-office');flush();admitted(false);
for(const callback of windowEvents.hashchange||[])callback({oldURL:mismatchOld,newURL:mismatchURL});assert.equal(find('#finance-tab-leave').getAttribute('aria-selected'),'true','mismatched HCM source does not override Policy selection');assert.equal(click(find('[data-finance-side="leave"]'),{ctrlKey:true}).defaultPrevented,true);
panel().setAttribute('data-leave-context',contextID);location.hash='';flush();admitted();click(find('#finance-tab-leave'));flush();
// A complete changed sample source on a new physical root resets local state.
const beforeSource=root(), nextSource=beforeSource.cloneNode(true), source=JSON.parse(nextSource.dataset.financeSource);assert.equal(nextSource.querySelector('#leave-sample-search').value,'cutoff');assert.equal(nextSource.querySelector('#leave-sample-backup').value,'seojun');assert.equal(nextSource.querySelector('[data-leave-detail="request"]').open,true);source[1]+=' · changed sample fixture';nextSource.setAttribute('data-finance-label',source[1]);nextSource.setAttribute('data-finance-source',JSON.stringify(source));beforeSource.replaceWith(nextSource);document.activeElement=tree;location.hash='';flush();admitted();
assert.equal(find('#leave-sample-search').value,'');assert.equal(find('#leave-sample-backup').value,'none');for(const detail of panel().querySelectorAll('[data-leave-detail]'))assert.equal(detail.open,false);assert.match(find('[data-leave-status]').textContent,/Sample preview reset/);
for(const label of ['Live leave approval unavailable','Live coverage assignment unavailable','Live timesheet locking unavailable'])assert.equal(panel().querySelectorAll('button').find(node=>node.textContent===label).disabled,true);
console.log('HCM current source local preview, independent SSR entries, refusal and remount PASS');
}
if(input.scenario==='metadata') {
  admitted();click(find('#finance-tab-leave'));const search=find('#leave-sample-search'),backup=find('#leave-sample-backup');search.value=' ';emit('input',{target:search});backup.value='seojun';emit('change',{target:backup});
  const compat=find('.console-app').append(new Element('a',{class:'rail-nav',href:'#leave-time'}));
  const iam=find('#identity-tenant')||tree.append(new Element('input',{id:'independent-iam-fixture'}));iam.focus();
  const problems=[],observations=[];const check=(value,label)=>{if(!value)problems.push(label);};
  for(const attribute of ['data-leave-source','data-leave-context'])for(const carrier of ['bound','never-bound']) {
    click(find('#finance-tab-leave'));flush();iam.focus();
    const expected=panel().getAttribute(attribute);
    if(carrier==='never-bound'){const replacement=root().cloneNode(true);root().replaceWith(replacement);replacement.querySelector('#finance-panel-leave').removeAttribute(attribute);}else panel().removeAttribute(attribute);
    flush();const detail=find('[data-leave-detail="request"]'),summary=detail.querySelector('summary');
    check(panel().querySelectorAll('#leave-sample-search, #leave-sample-backup, [data-finance-action], [data-finance-anchor-action], [data-leave-close]').every(node=>node.disabled),attribute+' '+carrier+' controls disabled');check(find('#finance-tab-leave').disabled,attribute+' '+carrier+' tab disabled');check(/unavailable.*controls disabled/.test(find('[data-leave-status]').textContent),attribute+' '+carrier+' refusal status');
    for(const modifiers of [{},{ctrlKey:true},{metaKey:true}]){detail.open=false;const event=click(summary,modifiers);check(event.defaultPrevented&&!detail.open,attribute+' '+carrier+' summary '+JSON.stringify(modifiers));observations.push({attribute,carrier,modifiers,prevented:event.defaultPrevented,open:detail.open});}
    for(const control of [find('[data-finance-side="leave"]'),find('a.rail-nav[href="#finance-panel-leave"]'),compat])for(const modifiers of [{},{metaKey:true}])check(click(control,modifiers).defaultPrevented,attribute+' '+carrier+' authored entry '+control.getAttribute('href')+' '+JSON.stringify(modifiers));
    location.hash='';
    panel().setAttribute(attribute,expected);flush();admitted();assert.equal(find('#leave-sample-search').value,' ');assert.equal(find('#leave-sample-backup').value,'seojun');check(document.activeElement===iam,attribute+' '+carrier+' outside IAM focus remains: '+document.activeElement.id);
    for(const modifiers of [{},{metaKey:true}]){detail.open=false;assert.equal(click(summary,modifiers).defaultPrevented,false);assert.equal(detail.open,true,'healthy native summary default admitted');detail.open=false;emit('toggle',{target:detail});}
    for(const side of ['vendors','billing'])check(!click(find(`[data-finance-side="${side}"]`),{ctrlKey:true}).defaultPrevented,attribute+' '+carrier+' neighbor '+side);location.hash='';
  }
  console.log(JSON.stringify({scenario:input.scenario,observations,problems}));assert.deepEqual(problems,[]);
}
if(input.scenario==='fragments') {
  admitted();assert.equal(find('[data-leave-detail="request"]').open,true,'initial direct detail entry');assert.equal(document.activeElement.id,'leave-sample-request-summary');
  const iam=find('#identity-tenant')||tree.append(new Element('input',{id:'independent-iam-fixture'})),problems=[],observations=[];
  for(const name of ['hashchange','popstate'])for(const fragment of ['', '#', '#%', '#%5B', '#finance-panel-leave%2Efinance-panel', '#finance-panel-leave%2C%23missing']) {
    ledger();iam.focus();location.hash=fragment;const exceptions=[];for(const callback of windowEvents[name]||[])try{callback({});}catch(error){exceptions.push(error.name+': '+error.message);}flush();
    const selected=find('#finance-tab-leave').getAttribute('aria-selected')==='true',moved=document.activeElement!==iam;observations.push({name,fragment,exceptions,selected,moved});if(exceptions.length||selected||moved)problems.push(name+' '+fragment);
  }
  const unusual=tree.append(new Element('div',{id:'finance-panel-leave.finance-panel'}));assert.ok(document.getElementById(unusual.id)===unusual,'literal platform ID lookup');ledger();iam.focus();location.hash='#finance-panel-leave%2Efinance-panel';windowEvents.hashchange.at(-1)({});flush();if(find('#finance-tab-leave').getAttribute('aria-selected')==='true'||document.activeElement!==iam)problems.push('outside literal ID is not Leave');unusual.remove();
  for(const name of ['hashchange','popstate']) { ledger();iam.focus();const duplicate=tree.append(new Element('div',{id:'leave-sample-request'}));location.hash='#leave-sample-request';windowEvents[name].at(-1)({});flush();admitted(false);assert.ok(document.activeElement===iam,'duplicate destination refused by HCM');duplicate.remove();location.hash='';flush();admitted(); }
  ledger();location.hash='#finance-panel-leave';for(const callback of windowEvents.hashchange||[])callback({});flush();assert.equal(find('#finance-tab-leave').getAttribute('aria-selected'),'true');
  location.hash='#leave-sample-request';for(const callback of windowEvents.popstate||[])callback({});flush();assert.equal(document.activeElement.id,'leave-sample-request-summary');
  console.log(JSON.stringify({scenario:input.scenario,observations,problems}));assert.deepEqual(problems,[]);
}
if(input.scenario==='reset') {
  admitted();click(find('#finance-tab-leave'));const search=find('#leave-sample-search'),backup=find('#leave-sample-backup');search.value='cutoff';emit('input',{target:search});backup.value='seojun';emit('change',{target:backup});for(const detail of panel().querySelectorAll('[data-leave-detail]')){detail.open=true;emit('toggle',{target:detail});}
  const iam=find('#identity-tenant')||tree.append(new Element('input',{id:'independent-iam-fixture'}));iam.focus();const original=root(),carrier=original.cloneNode(true),source=JSON.parse(carrier.dataset.financeSource);
  assert.equal(carrier.querySelector('#leave-sample-search').value,'cutoff','genuine dirty search carrier');assert.equal(carrier.querySelector('#leave-sample-backup').value,'seojun','genuine dirty selection carrier');assert.ok(carrier.querySelectorAll('[data-leave-detail]').every(detail=>detail.open),'genuine open detail carrier');
  source[1]+=' · changed sample fixture';carrier.setAttribute('data-finance-label',source[1]);carrier.setAttribute('data-finance-source',JSON.stringify(source));original.replaceWith(carrier);location.hash='';flush();admitted();
  const observation={query:find('#leave-sample-search').value,backup:find('#leave-sample-backup').value,details:panel().querySelectorAll('[data-leave-detail]').map(detail=>detail.open),count:find('#leave-sample-count').textContent,coverage:find('#leave-coverage-preview').textContent,status:find('[data-leave-status]').textContent};console.log(JSON.stringify({scenario:input.scenario,observation}));
  assert.equal(observation.query,'');assert.equal(observation.backup,'none');assert.deepEqual(observation.details,[false,false,false]);assert.match(observation.count,/3 sample items/);assert.match(observation.coverage,/No sample backup selected/);assert.match(observation.status,/Sample preview reset/);assert.ok(document.activeElement===iam,'outside IAM focus remains');
  for(const label of ['Live leave approval unavailable','Live coverage assignment unavailable','Live timesheet locking unavailable'])assert.equal(panel().querySelectorAll('button').find(node=>node.textContent===label).disabled,true);
}
"###;

#[test]
fn leave_preview_current_source_controls_entries_refusal_and_lifetime() {
    run_case("current");
}

#[test]
fn leave_preview_current_missing_source_metadata_refuses_native_defaults() {
    run_case("metadata");
}

#[test]
fn leave_preview_current_fragments_resolve_literal_unique_ids() {
    run_case("fragments");
}

#[test]
fn leave_preview_current_changed_source_resets_carried_local_state() {
    run_case("reset");
}

fn run_case(scenario: &str) {
    let owner = Owner::new();
    let html = owner.with(|| super::super::document::ServerShell().into_view().to_html());
    owner.cleanup();
    let input = serde_json::json!({"scenario": scenario, "html": html, "tabs": crate::app::ssr_tab_navigation_script(), "finance": super::super::finance_preview::SCRIPT, "policy": super::super::policy_navigation::SCRIPT, "bridge": super::script()});
    let mut child = Command::new("node")
        .args(["-e", CHECK])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node runs current Leave source proof");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
