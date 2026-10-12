pub(super) const SCRIPT: &str = r#"
const assert = require('node:assert/strict'), vm = require('node:vm');
const input = JSON.parse(require('node:fs').readFileSync(0, 'utf8'));
class Element {
  constructor(tag='div') { this.tagName=tag; this.children=[]; this.dataset={}; this.events={}; this.attrs={}; this.hidden=false; this._value=''; this.isConnected=true; if(['input','select','button'].includes(tag))this.disabled=false; }
  append(...children) { for (const child of children) { child.parentElement=this; child.isConnected=this.isConnected; this.children.push(child); } }
  replaceChildren(...children) { if (this.children.some(child=>child.contains(document.activeElement))) document.activeElement=document.body; for (const child of this.children) { child.parentElement=null; child.isConnected=false; } this.children=[]; this.append(...children); }
  setAttribute(name,value) { this.attrs[name]=String(value); }
  get value() { return this.tagName==='select' ? (this.children.some(c=>c.value===this._value) ? this._value : this.children[0]?.value || '') : this._value; }
  set value(value) { this._value=String(value); }
  get selectedOptions() { return this.children.filter(c=>c.value===this.value); }
  get disabled() { return this._disabled === true; }
  set disabled(value) { this._disabled=Boolean(value);if(this._disabled&&document.activeElement===this)document.activeElement=document.body; }
  addEventListener(name,action) { (this.events[name] ||= []).push(action); }
  emit(name) { for (const action of this.events[name] || []) action({target:this,preventDefault(){}}); }
  contains(target) { for(let node=target;node;node=node.parentElement) if(node===this)return true;return false; }
  closest(selector) { for(let node=this;node;node=node.parentElement)if(selector==='[data-live-results]'&&node.dataset.liveResults)return node;return null; }
  querySelectorAll(selector) { const result=[];for(const child of this.children){if(selector==='[data-live-object]'&&child.dataset.liveObject||selector==='[data-live-fact]'&&child.dataset.liveFact)result.push(child);result.push(...child.querySelectorAll(selector));}return result; }
  focus(options) { if(this.disabled)return;this.focusOptions=options; document.activeElement=this; }
}
const elements={}, panel=new Element('section');
function add(id,tag='div',parent=panel) { const e=new Element(tag);e.id=id;elements[id]=e;parent.append(e);return e; }
add('oyatie-sign-in');
for(const id of ['tenant','account'])add('sign-in-'+id,'input');
for(const id of ['status'])add('sign-in-'+id);
for(const id of ['passkey-sign-in','session-restore','passkey-enroll','session-logout','passkey-cancel','ontology-refresh'])add(id,'button');
for(const id of ['passkey-form','passkey-invitation','verified-ontology-identity','verified-ontology-updated'])add(id);
const card=add('verified-ontology-card');for(const key of ['policyVersion','servedTenants','projectionLag','poisonedEntries']){const e=new Element();e.dataset.liveFact=key;card.append(e);}
const root=add('live-ontology-explorer','section');root.hidden=true;
for(const id of ['identity','query','status','page','detail','detail-title','properties','history','audit','audit-rows'])add('live-ontology-'+id,'div',root);
const schema=add('live-ontology-schema','section',root);schema.hidden=true;
for(const id of ['schema-title','schema-summary','schema-properties'])add('live-ontology-'+id,'div',schema);
for(const id of ['type','comparison','kind'])add('live-ontology-'+id,'select',root);
for(const id of ['revision','limit','property','from','to'])add('live-ontology-'+id,'input',root);
for(const id of ['read','types-refresh','audit-read','history-read','next','first'])add('live-ontology-'+id,'button',root);
for(const id of ['page','detail','audit','schema'])elements['live-ontology-'+id].dataset.liveResults='true';
for(const [id,values] of [['comparison',['none','equals','range']],['kind',['text','int','bool']]])for(const value of values){const option=new Element('option');option.value=value;elements['live-ontology-'+id].append(option);}
elements['live-ontology-revision'].value='1';elements['live-ontology-limit'].value='25';
const document={activeElement:panel,body:panel,visibilityState:'visible',getElementById:id=>elements[id],createElement:tag=>new Element(tag),addEventListener(){}};
let intervals=[], timers=[], calls=[], held=null, holdPath=null, nativeError=null, headerUser='usr_alice', pageEmpty=false;
const session={tenantId:'ten_acme',userId:'usr_alice',expiresAt:Math.floor(Date.now()/1000)+3600,roles:['foundry-operator']};
const object={object_ref:'ent_alpha',written_revision:1,upcast_state:'current',properties:{name:{value_type:'string',data_class:'INTERNAL_ONLY',value:'<script>Ada</script>'}}};
const schemaFixture=[
  {entity_type:'ety_record',revision:1,properties:[{name:'name',tier:'scalar',data_class:'INTERNAL_ONLY',required:true,value_type:null},{name:'note',tier:'scalar',data_class:'INTERNAL_ONLY',required:false,value_type:null}],primary_key_property:null,title_property:'name'},
  {entity_type:'ety_profile',revision:3,properties:[{name:'id',tier:'scalar',data_class:'PUBLIC',required:true,value_type:{kind:'integer'}},{name:'<script>label</script>',tier:'scalar',data_class:'PII_IDENTIFYING',required:false,value_type:{kind:'string'}},{name:'profile',tier:'struct',data_class:'INTERNAL_ONLY',required:false,value_type:{kind:'struct',fields:[{name:'flags',required:true,value_type:{kind:'array',element:{kind:'boolean'}}}]}}],primary_key_property:'id',title_property:'<script>label</script>'},
];
let nativeSchemas=schemaFixture;
function payload(url) {
  if(url.endsWith('identity-config'))return {enabled:true};
  if(url.endsWith('/session'))return session;
  if(url.includes('ontology-status'))return {policyVersion:'p',servedTenants:1,projectionLag:0,poisonedEntries:0};
  if(url.endsWith('/types'))return [{entity_type:'ety_record',revision:1,properties:['name']}];
  if(url.endsWith('/schemas'))return nativeSchemas;
  if(url.includes('/objects?'))return {requested_revision:Number(new URL('https://shell.invalid'+url).searchParams.get('revision')),objects:pageEmpty?[]:[object],next:null};
  if(url.endsWith('/history'))return [{ordinal:1,object_sequence:1,principal_id:'usr_alice',decision_id:'dec_native',action_type:'aty_record_write',audit_event_type:'record.written',schema_revision:1}];
  if(url.endsWith('/audit'))return [{ordinal:1,object_ref:'ent_alpha',action_type:'aty_record_write',disposition:'applied',poison_reason:null,principal_id:'usr_alice',decision_id:'dec_native'}];
  if(url.includes('/objects/'))return {requested_revision:Number(new URL('https://shell.invalid'+url).searchParams.get('revision')),object};
  return null;
}
function reply(url,error=nativeError) { const isRead=url.includes('/api/ontology/');return {ok:!isRead||!error,status:url.endsWith('session:logout')?204:isRead&&error?403:200,headers:{get:name=>({'x-oyatie-user':headerUser,'x-oyatie-tenant':session.tenantId,'x-oyatie-session-expires':String(session.expiresAt)})[name]??null},json:async()=>isRead&&error?{error}:payload(url)}; }
const context={document,Element,URLSearchParams,AbortController,window:{isSecureContext:true,PublicKeyCredential:function(){}},navigator:{credentials:{},locks:{request:(_,action)=>action()}},fetch:(url,options)=>{calls.push([url,options]);if(holdPath&&url.includes(holdPath)){holdPath=null;return new Promise(resolve=>{held=()=>resolve(reply(url));});}return Promise.resolve(reply(url));},setTimeout:callback=>{timers.push(callback);return timers.length;},clearTimeout(){},setInterval:callback=>intervals.push(callback)};
vm.runInNewContext(input.script,context,{timeout:1000});
const find=id=>elements[id], text=node=>(node.textContent||'')+node.children.map(text).join(' ');
async function flush() { for(let i=0;i<80;i++)await Promise.resolve(); }
async function restore() { find('session-restore').emit('click');await flush(); }
"#;
