pub(super) const SCRIPT: &str = r#"
const assert = require('node:assert/strict'), vm = require('node:vm');
const input = JSON.parse(require('node:fs').readFileSync(0, 'utf8'));
class Element {
  constructor(tag='div') { this.tagName=tag; this.children=[]; this.dataset={}; this.events={}; this.attrs={}; this.hidden=false; this._value=''; this.isConnected=true; this.style={};this.classList={add(){}};this.clientWidth=640;this.clientHeight=480;this.scrollLeft=0;this.scrollTop=0; if(['input','select','button'].includes(tag))this.disabled=false; }
  append(...children) { for (const child of children) { if(child.parentElement)child.parentElement.children=child.parentElement.children.filter(node=>node!==child); child.parentElement=this; child.isConnected=this.isConnected; this.children.push(child); } }
  replaceChildren(...children) { if (this.children.some(child=>child.contains(document.activeElement))) document.activeElement=document.body; for (const child of this.children) { child.parentElement=null; child.isConnected=false; } this.children=[]; this.append(...children); }
  setAttribute(name,value) { this.attrs[name]=String(value); }
  get value() { return this.tagName==='select' ? (this.children.some(c=>c.value===this._value) ? this._value : this.children[0]?.value || '') : this._value; }
  set value(value) { this._value=String(value); }
  get selectedOptions() { return this.children.filter(c=>c.value===this.value); }
  get disabled() { return this._disabled === true; }
  set disabled(value) { this._disabled=Boolean(value);if(this._disabled&&document.activeElement===this)document.activeElement=document.body; }
  addEventListener(name,action) { (this.events[name] ||= []).push(action); }
  emit(name,values={}) { for (const action of this.events[name] || []) action({target:this,preventDefault(){},...values}); }
  scrollBy({left=0,top=0}) {this.scrollLeft+=left;this.scrollTop+=top;}
  get offsetLeft() {return Number.parseFloat(this.style.left)||0;}
  get offsetTop() {return Number.parseFloat(this.style.top)||0;}
  get offsetWidth() {return this.mockWidth||0;}
  get offsetHeight() {return this.mockHeight||0;}
  getBBox() {return this.mockBounds||{x:0,y:0,width:0,height:0};}
  contains(target) { for(let node=target;node;node=node.parentElement) if(node===this)return true;return false; }
  closest(selector) { for(let node=this;node;node=node.parentElement)if(selector==='[data-live-results]'&&node.dataset.liveResults||selector==='[hidden]'&&node.hidden)return node;return null; }
  getClientRects() { return this.isConnected && !this.closest('[hidden]') ? [{}] : []; }
  querySelectorAll(selector) { const result=[];for(const child of this.children){if(selector===child.tagName||selector==='[data-live-object]'&&child.dataset.liveObject||selector==='[data-live-fact]'&&child.dataset.liveFact||selector==='[data-model-type]'&&child.dataset.modelType||selector==='[data-model-link]'&&child.dataset.modelLink||selector==='[data-model-edge]'&&child.dataset.modelEdge)result.push(child);result.push(...child.querySelectorAll(selector));}return result; }
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
for(const id of ['access-notice','model-note','model','model-summary','library-summary','library-types','library-links','model-table','graph-help','graph-controls','zoom','inspector-empty'])add('live-ontology-'+id,'div',root);
const graph=add('live-ontology-graph','div',root),stage=add('live-ontology-stage','div',graph);add('live-ontology-scene','div',stage);
const link=add('live-ontology-link','section',root);link.hidden=true;add('live-ontology-link-properties','div',link);
add('live-ontology-objects','section',root);add('live-ontology-search','input',root);
for(const id of ['mode-schema','mode-objects','view-graph','view-table','zoom-out','zoom-in','fit'])add('live-ontology-'+id,'button',root);
const schema=add('live-ontology-schema','section',root);schema.hidden=true;
for(const id of ['schema-title','schema-summary','schema-properties'])add('live-ontology-'+id,'div',schema);
for(const id of ['type','comparison','kind'])add('live-ontology-'+id,'select',root);
for(const id of ['revision','limit','property','from','to'])add('live-ontology-'+id,'input',root);
for(const id of ['read','types-refresh','audit-read','history-read','next','first'])add('live-ontology-'+id,'button',root);
elements['live-ontology-objects'].append(...['query','page','detail','audit','first','next'].map(id=>elements['live-ontology-'+id]));
elements['live-ontology-detail'].append(...['detail-title','properties','history','history-read'].map(id=>elements['live-ontology-'+id]));
elements['live-ontology-audit'].append(elements['live-ontology-audit-rows']);
elements['live-ontology-model'].append(...['graph','graph-help','graph-controls','model-table'].map(id=>elements['live-ontology-'+id]));
for(const id of ['page','detail','audit','schema'])elements['live-ontology-'+id].dataset.liveResults='true';
for(const [id,values] of [['comparison',['none','equals','range']],['kind',['text','int','bool']]])for(const value of values){const option=new Element('option');option.value=value;elements['live-ontology-'+id].append(option);}
elements['live-ontology-revision'].value='1';elements['live-ontology-limit'].value='25';
for(const id of ['draft-controls','draft-inspector','draft-tray','draft-state','draft-status','draft-selection','draft-diff','draft-diagnostics','property-compatibility'])add('live-ontology-'+id,'div',root);
for(const id of ['draft-choice','property-choice','property-kind','property-required','new-property-kind','new-property-required','link-from','link-to','link-cardinality','new-link-from','new-link-to','new-link-cardinality'])add('live-ontology-'+id,'select',root);
for(const id of ['entity-name','entity-revision','property-name','new-property-name','link-revision','new-entity-id','new-entity-name','new-link-id'])add('live-ontology-'+id,'input',root);
for(const id of ['draft-more','draft-new','draft-save','draft-reload','draft-validate'])add('live-ontology-'+id,'button',root);
for(const id of ['entity-edit','property-edit','property-add','link-edit','entity-add','link-add']){const form=add('live-ontology-'+id,'form',findEditorParent());form.append(new Element('button'));}
function findEditorParent(){return elements['live-ontology-draft-inspector'];}
add('live-ontology-draft-inspector-title','h4',findEditorParent()).setAttribute('tabindex','-1');findEditorParent().append(elements['live-ontology-property-choice']);
for(const [form,fields] of [['entity-edit',['entity-name','entity-revision']],['property-edit',['property-name','property-kind','property-required']],['link-edit',['link-from','link-to','link-cardinality','link-revision']]])elements['live-ontology-'+form].append(...fields.map(id=>elements['live-ontology-'+id]));
for(const [id,values] of [['property-kind',['legacy','string','integer','double','boolean','date','timestamp']],['property-required',['required','optional']],['new-property-kind',['string','integer','double','boolean','date','timestamp']],['new-property-required',['optional','required']],['link-cardinality',['one_to_one','one_to_many','many_to_many']],['new-link-cardinality',['one_to_many','one_to_one','many_to_many']]])for(const value of values){const option=new Element('option');option.value=value;elements['live-ontology-'+id].append(option);}
const document={activeElement:panel,body:panel,visibilityState:'visible',getElementById:id=>elements[id],createElement:tag=>new Element(tag),createElementNS:(namespace,tag)=>{const e=new Element(tag);e.namespaceURI=namespace;return e;},addEventListener(){}};
let intervals=[], timers=[], calls=[], held=null, holdPath=null, nativeError=null, statusError=null, modelEnabled=false, modelError=null, modelLoseAck=false, modelTamper=false, modelValidation=null, headerUser='usr_alice', pageEmpty=false;
const session={tenantId:'ten_acme',userId:'usr_alice',expiresAt:Math.floor(Date.now()/1000)+3600,roles:['foundry-operator']};
const object={object_ref:'ent_alpha',written_revision:1,upcast_state:'current',properties:{name:{value_type:'string',data_class:'INTERNAL_ONLY',value:'<script>Ada</script>'}}};
const schemaFixture=[
  {entity_type:'ety_record',revision:1,properties:[{name:'name',tier:'scalar',data_class:'INTERNAL_ONLY',required:true,value_type:null},{name:'note',tier:'scalar',data_class:'INTERNAL_ONLY',required:false,value_type:null}],primary_key_property:null,title_property:'name'},
  {entity_type:'ety_profile',revision:3,properties:[{name:'id',tier:'scalar',data_class:'PUBLIC',required:true,value_type:{kind:'integer'}},{name:'<script>label</script>',tier:'scalar',data_class:'PII_IDENTIFYING',required:false,value_type:{kind:'string'}},{name:'profile',tier:'struct',data_class:'INTERNAL_ONLY',required:false,value_type:{kind:'struct',fields:[{name:'flags',required:true,value_type:{kind:'array',element:{kind:'boolean'}}}]}}],primary_key_property:'id',title_property:'<script>label</script>'},
];
const labelFixture=[{entity_type:'ety_record',revision:1,display_name:'Record'}, {entity_type:'ety_profile',revision:3,display_name:'직원 <img src=x onerror=alert(1)>'}];
let nativeSchemas=schemaFixture, nativeLinks=[], nativeLabels=null;
function payload(url) {
  if(url.endsWith('identity-config'))return {enabled:true};
  if(url.endsWith('/session'))return session;
  if(url.includes('ontology-status'))return {policyVersion:'p',servedTenants:1,projectionLag:0,poisonedEntries:0};
  if(url.endsWith('/types'))return [{entity_type:'ety_record',revision:1,properties:['name']}];
  if(url.endsWith('/schemas'))return nativeSchemas;
  if(url.endsWith('/type-labels'))return nativeLabels ?? labelFixture.filter(label=>nativeSchemas.some(schema=>schema.entity_type===label.entity_type)).map(label=>({...label,revision:nativeSchemas.find(schema=>schema.entity_type===label.entity_type).revision}));
  if(url.endsWith('/link-types'))return nativeLinks;
  if(url.includes('/objects?'))return {requested_revision:Number(new URL('https://shell.invalid'+url).searchParams.get('revision')),objects:pageEmpty?[]:[object],next:null};
  if(url.endsWith('/history'))return [{ordinal:1,object_sequence:1,principal_id:'usr_alice',decision_id:'dec_native',action_type:'aty_record_write',audit_event_type:'record.written',schema_revision:1}];
  if(url.endsWith('/audit'))return [{ordinal:1,object_ref:'ent_alpha',action_type:'aty_record_write',disposition:'applied',poison_reason:null,principal_id:'usr_alice',decision_id:'dec_native'}];
  if(url.includes('/objects/'))return {requested_revision:Number(new URL('https://shell.invalid'+url).searchParams.get('revision')),object};
  return null;
}
const modelDocument={entities:[{id:'ety_record',display_name:'Record',properties:[{name:'name',tier:'scalar',data_class:'InternalOnly',required:true,value_type:null,display:null},{name:'structure',tier:'struct',data_class:'InternalOnly',required:false,value_type:{shape:'struct',fields:[{name:'date',required:true,value_type:{shape:'scalar',kind:'date'}}]},display:{display_name:'Untouched',description:null,icon:'native',color:null,plural_name:null}}],revision:1,pillar:null,primary_key_property:null,title_property:'name',display:{display_name:'Record',description:'Preserve this',icon:null,color:null,plural_name:null}}],links:[]};
const modelWorkspace={generation:'9007199254740993',document:modelDocument,capabilities:{author:true,publish:false}}, modelDrafts=new Map();let nextDraft=9007199254740993n;
function modelReply(url,options){
  let body=null,code=modelEnabled?modelError:'forbidden';if(code)return {ok:false,status:code==='unauthenticated'?401:code==='model_unavailable'?503:403,json:async()=>({error:code})};
  const path=new URL('https://shell.invalid'+url).pathname, payload=options.body?JSON.parse(options.body):null;
  if(path.endsWith('/model'))body=modelWorkspace;
  else if(path.endsWith('/drafts') && options.method==='POST'){const id=String(nextDraft++);body={id,tenant_id:session.tenantId,owner:session.userId,base_generation:payload.expected_generation,version:'1',document:payload.document,publication:null};modelDrafts.set(id,structuredClone(body));}
  else if(path.endsWith('/drafts'))body={drafts:[...modelDrafts.values()].filter(d=>d.owner===session.userId).map(d=>({id:d.id,version:d.version,base_generation:d.base_generation,published_generation:null})),next:null};
  else {const parts=path.split('/'),id=parts.at(-1)==='save'||parts.at(-1)==='validate'?parts.at(-2):parts.at(-1),draft=modelDrafts.get(id);
    if(!draft || draft.owner!==session.userId)return {ok:false,status:404,json:async()=>({error:'draft_not_found'})};
    if(payload && (payload.expected_version!==draft.version || payload.expected_base!==draft.base_generation))return {ok:false,status:409,json:async()=>({error:'draft_conflict'})};
    if(path.endsWith('/save')){draft.version=String(BigInt(draft.version)+1n);draft.document=structuredClone(payload.document);body=draft;}
    else if(path.endsWith('/validate'))body={draft_id:id,draft_version:draft.version,base_generation:draft.base_generation,valid:true,diagnostics:[],...(modelValidation || {})};
    else body=draft;
  }
  if(options.method==='POST' && modelLoseAck){modelLoseAck=false;throw new Error('network lost after native commit');}
  body=structuredClone(body);if(modelTamper && options.method==='POST' && body.id){modelTamper=false;body.version='9007199254740995';}
  return {ok:true,status:options.method==='POST' && path.endsWith('/drafts')?201:200,headers:{get:name=>({'x-oyatie-user':headerUser,'x-oyatie-tenant':session.tenantId,'x-oyatie-session-expires':String(session.expiresAt)})[name]??null},json:async()=>body};
}
function reply(url,error=nativeError) { const isRead=url.includes('/api/ontology/');if(url.includes('ontology-status') && statusError)return {ok:false,status:statusError,json:async()=>({error:'forbidden'})};return {ok:!isRead||!error,status:url.endsWith('session:logout')?204:isRead&&error?403:200,headers:{get:name=>({'x-oyatie-user':headerUser,'x-oyatie-tenant':session.tenantId,'x-oyatie-session-expires':String(session.expiresAt)})[name]??null},json:async()=>isRead&&error?{error}:payload(url)}; }
const context={document,Element,URLSearchParams,AbortController,TextEncoder,getComputedStyle:element=>({strokeWidth:String(element.mockStrokeWidth||0)}),window:{isSecureContext:true,PublicKeyCredential:function(){}},navigator:{credentials:{},locks:{request:(_,action)=>action()}},fetch:(url,options)=>{calls.push([url,options]);if(url.includes('ontology-status') && statusError==='network')return Promise.reject(new Error('status transport unavailable'));if(url.includes('/model')){if(holdPath && url.includes(holdPath)){holdPath=null;return new Promise(resolve=>{const captured=modelReply(url,options);held=()=>resolve(captured);});}try{return Promise.resolve(modelReply(url,options));}catch(e){return Promise.reject(e);}}if(holdPath&&url.includes(holdPath)){holdPath=null;return new Promise(resolve=>{held=()=>resolve(reply(url));});}return Promise.resolve(reply(url));},setTimeout:callback=>{timers.push(callback);return timers.length;},clearTimeout(){},setInterval:callback=>intervals.push(callback)};
vm.runInNewContext(input.script,context,{timeout:1000});
const find=id=>elements[id], text=node=>(node.textContent||'')+node.children.map(text).join(' ');
async function flush() { for(let i=0;i<80;i++)await Promise.resolve(); }
async function restore() { find('session-restore').emit('click');await flush(); }
"#;
