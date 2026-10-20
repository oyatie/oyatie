pub(super) const SCRIPT: &str = r#"
function revealOntologyDetails(element, root) {
  if(!element?.isConnected || !root.contains(element) || element.closest('[hidden]'))return false;
  for(let owner=element;owner && owner!==root;owner=owner.parentElement)if(owner.tagName.toLowerCase()==='details')owner.open=true;
  return true;
}
function createOntologyModelView(selectType, fallback, selectRelationship = () => {}, modeChanged = () => {}) {
  const find = id => document.getElementById(`live-ontology-${id}`);
  let values=[],catalogAvailable=false;
  let schemas = [], links = [], selectedType = null, selectedLink = null, source = 'Published declarations';
  let representation = 'graph', zoom = 1, width = 640, height = 360;
  let scope='related',context=null,heldViewport=null,heldSelection=null,relatedCount=null,drawn=null,framePending=true;
  let cameraManual=false,pointerScroll=false,cameraViewport=null,revealEpoch=0;
  const projection=()=>ontologyModelProjection(schemas,links,{type:selectedType,link:selectedLink},scope);
  let typeButtons = [], linkButtons = [], graphLabels = [], graphNodes = [], graphEdges = [];
  const arrowSize = 8, arrowRefX = 7, arrowRefY = 4;
  const cardinality = value => ({one_to_one:'one → one',one_to_many:'one → many',many_to_many:'many → many'})[value];
  const cardinalityLimits = value => ({
    one_to_one:'Each source can link to at most one target; each target can link from at most one source.',
    one_to_many:'Each source can link to many targets; each target can link from at most one source.',
    many_to_many:'Each source can link to many targets; each target can link from many sources.',
  })[value] + ' No links are required.';
  function node(tag, text = '', className = '') {
    const element = document.createElement(tag); element.textContent = text; element.className = className; return element;
  }
  function button(text, action, className = '') {
    const element = node('button', text, className); element.type = 'button'; element.addEventListener('click', action); return element;
  }
  function focusedDeclaration() {
    const element=document.activeElement,surface=['scene','library-types','library-links','library-values','model-table'].find(id=>find(id).contains(element));
    if(surface && (element?.dataset.modelType || element?.dataset.modelLink))return {element,surface,type:element.dataset.modelType,link:element.dataset.modelLink};
    const value=surface==='library-values'?element.closest('details'):null;if(value?.dataset.modelValue!==undefined && element===value.querySelector('summary'))return {element,surface,valueIndex:Number(value.dataset.modelValue),valueKey:JSON.stringify(values[Number(value.dataset.modelValue)])};
    const disclosure=['library-outside','library-invalid'].find(id=>find(id)?.querySelector('summary')===element);return disclosure?{element,surface:'library-types',disclosure}:null;
  }
  function restoreDeclaration(saved, same=true) {
    if(!saved || document.activeElement!==document.body && document.activeElement!==saved.element || saved.element.isConnected)return;
    const buttons=saved.type?typeButtons:linkButtons;
    const candidate=same?(saved.valueKey?JSON.stringify(values[saved.valueIndex])===saved.valueKey?Array.from(find('library-values').querySelectorAll('[data-model-value]')).find(row=>Number(row.dataset.modelValue)===saved.valueIndex)?.querySelector('summary'):null:saved.disclosure?find(saved.disclosure)?.querySelector('summary'):buttons.find(([element,id])=>id===(saved.type ?? saved.link) && find(saved.surface).contains(element))?.[0]):null;
    const replacement=candidate?.isConnected && !candidate.disabled && candidate.getClientRects().length?candidate:null;
    const local=saved.surface.startsWith('library')?find('search'):find(representation==='graph'?'graph':'view-table');
    (replacement || local)?.focus({preventScroll:true});
  }
  function declarationButton(row, type, className) {
    const id=type?row.entity_type:row.link_type;
    const element=button('',()=>{
      if(element.disabled || !element.isConnected || !find('explorer').contains(element))return;
      const owned=focusedDeclaration(),prior=context,accepted=type?selectType(id):selectRelationship(id);
      if(accepted===false || type && accepted!==true)return;
      if(prior!==context)return;
      if(type)select(id);else selectLink(id);
      if(accepted && typeof accepted==='object')renderOntologySelectedContext(accepted);
      restoreDeclaration(owned);
    },className);
    const unique=projection()[type?'typeById':'linkById'].has(id);
    element.disabled=!unique;
    if(!unique)element.setAttribute('aria-label',`Missing or ambiguous ${type?'type':'relationship'} ID ${id}; inspect the complete Table and validate the draft.`);
    if(type){element.append(node('span',row.display_name),node('small',` · ${id}`));element.dataset.modelType=id;typeButtons.push([element,id]);}
    else{element.textContent=id;element.dataset.modelLink=id;linkButtons.push([element,id]);}
    if(className==='foundry-library-item'){
      const name=(type?row.display_name:row.display?.display_name) || id,rows=type?schemas:links,label=other=>type?other.display_name:other.display?.display_name || other.link_type;
      const identity=node('small',id,rows.filter(other=>label(other)===name).length>1?'':'sr-only');element.textContent='';element.replaceChildren(node('span',name),identity);element.setAttribute('title',`${name} · ${id} · revision ${row.revision}`);if(unique)element.setAttribute('aria-label',`Inspect ${type?'object type':'relationship'} ${name} · ${id} · revision ${row.revision}`);
    }
    return element;
  }
  const typeButton=(schema,className='')=>declarationButton(schema,true,className);
  const linkButton=(link,className='')=>declarationButton(link,false,className);
  function selection() {
    for (const [element, id] of typeButtons) element.setAttribute('aria-pressed', String(!selectedLink && selectedType === id));
    for (const [element, id] of linkButtons) element.setAttribute('aria-pressed', String(selectedLink === id));
  }
  function selectLink(id) {
    const link = projection().linkById.get(id); if (!link) return false;
    const prior=projection().schemas,changed=selectedLink!==id;if(changed){relatedCount=null;zoom=1;framePending=true;}
    const active = document.activeElement, restore = find('schema').contains(active);
    selectedLink = id; selectedType=null;if(changed)refreshSelection(prior);find('schema').hidden = true;
    find('link-properties').replaceChildren(ontologyTable('Relationship declaration', ['Declaration', 'Value'], [
      ['Relationship type', link.link_type], ['From entity type (source)', typeText(link.from_entity_type)], ['To entity type (target)', typeText(link.to_entity_type)],
      ['Cardinality', `${cardinality(link.cardinality)} (${link.cardinality})`], ['Limits per relationship type and tenant', cardinalityLimits(link.cardinality)],
      ['Cross-tenant intent', String(link.allow_cross_tenant)], ['Declaration revision', link.revision],
    ]));
    find('link').hidden = false; find('inspector-empty').hidden = true;renderOntologySelectedContext({source,kind:'relationship',linkId:link.link_type,from:typeContext(link.from_entity_type),to:typeContext(link.to_entity_type)}); selection();
    if (restore) fallback();return true;
  }
  function typeText(id) { return `${typeContext(id).name} · ${id}`; }
  function typeContext(id) { return {id,name:projection().typeById.get(id)?.display_name || id}; }
  function select(id) {
    if(id!==null && !projection().typeById.has(id))return false;
    const prior=projection().schemas,changed=selectedType!==id || selectedLink!==null;if(changed){relatedCount=null;zoom=1;framePending=true;}
    const restore = find('link').contains(document.activeElement);
    selectedType = id; selectedLink = null;if(changed)refreshSelection(prior); find('link-properties').replaceChildren(); find('link').hidden = true;
    renderOntologySelectedContext(schemas.some(row=>row.entity_type===id)?{source,kind:'type',type:typeContext(id)}:null);
    find('inspector-empty').hidden = Boolean(id); selection(); if (restore) fallback();return true;
  }
  function library(retaining=false) {
    const owned=focusedDeclaration();
    typeButtons=typeButtons.filter(([element])=>!find('library-types').contains(element));
    linkButtons=linkButtons.filter(([element])=>!find('library-links').contains(element));
    renderOntologyResourceLibrary(find,node,typeButton,linkButton,schemas,links,values,projection(),scope,catalogAvailable);
    selection();if(!retaining)restoreDeclaration(owned);
  }
  function refreshSelection(prior){refreshGraph();const current=projection().schemas;if(current.length!==prior.length || current.some((row,index)=>row!==prior[index]))library();}
  function scale(value) {
    const active = document.activeElement;
    zoom = Math.max(0.25, Math.min(2, value));
    find('scene').style.transform = `scale(${zoom})`;
    find('stage').style.width = `${width * zoom}px`; find('stage').style.height = `${height * zoom}px`;
    for (const label of graphLabels) label.style.transform = `scale(${1 / zoom})`;
    find('zoom').textContent = `${Math.round(zoom * 100)}%`;
    find('zoom-out').disabled = zoom <= 0.25; find('zoom-in').disabled = zoom >= 2;
    if ([find('zoom-out'),find('zoom-in')].includes(active) && active.disabled) find('fit').focus({preventScroll:true});
  }
  function occupied(overview=false) {
    const selected=selectedLink?projection().linkById.get(selectedLink):null;
    const ids=new Set(selected?[selected.from_entity_type,selected.to_entity_type]:[selectedType]);
    const cards=overview?graphNodes:graphNodes.filter(card=>ids.has(card.dataset.modelType));
    const edges=overview?graphEdges:graphEdges.filter(edge=>edge.dataset.modelEdge===selectedLink);
    const labels=overview?graphLabels:graphLabels.filter(label=>label.dataset.modelLink===selectedLink);
    const boxes=cards.map(card=>({x:card.offsetLeft,y:card.offsetTop,w:card.offsetWidth,h:card.offsetHeight}));
    for(const edge of edges){const box=edge.getBBox(),padding=Math.hypot(arrowRefX,arrowRefY)*Number.parseFloat(getComputedStyle(edge).strokeWidth);boxes.push({x:box.x-padding,y:box.y-padding,w:box.width+2*padding,h:box.height+2*padding});}
    for(const label of labels)boxes.push({x:label.offsetLeft,y:label.offsetTop,w:label.offsetWidth/zoom,h:label.offsetHeight/zoom});
    if(!boxes.length)return null;
    const left=Math.min(...boxes.map(box=>box.x)),top=Math.min(...boxes.map(box=>box.y));
    return {left,top,width:Math.max(...boxes.map(box=>box.x+box.w))-left,height:Math.max(...boxes.map(box=>box.y+box.h))-top};
  }
  function frameSelection(onlyIfHidden=false, focused=null) {
    const graph=find('graph'),bounds=occupied();if(!bounds)return;
    const first=graphNodes.find(card=>card.dataset.modelType===(selectedType || projection().linkById.get(selectedLink)?.from_entity_type));
    const focus=focused?.surface==='scene'?(focused.type?graphNodes.find(card=>card.dataset.modelType===focused.type):graphLabels.find(label=>label.dataset.modelLink===focused.link)):null;
    const target=focus?{left:focus.offsetLeft,top:focus.offsetTop,width:focus.offsetWidth/(focused.link?zoom:1),height:focus.offsetHeight/(focused.link?zoom:1)}:bounds.width*zoom<=graph.clientWidth-32 && bounds.height*zoom<=graph.clientHeight-32?bounds:first?{left:first.offsetLeft,top:first.offsetTop,width:first.offsetWidth,height:first.offsetHeight}:bounds;
    if(!onlyIfHidden || target.left*zoom<graph.scrollLeft || target.top*zoom<graph.scrollTop || (target.left+target.width)*zoom>graph.scrollLeft+graph.clientWidth || (target.top+target.height)*zoom>graph.scrollTop+graph.clientHeight)restoreViewport({left:Math.max(0,(target.left+target.width/2)*zoom-graph.clientWidth/2),top:Math.max(0,(target.top+target.height/2)*zoom-graph.clientHeight/2)});
    framePending=false;cameraManual=false;
  }
  function fit(overview=false) {
    const graph=find('graph');if(!graphNodes.length)return;cameraManual=overview;
    const fits=()=>{const box=occupied(overview);return box && box.width*zoom<=graph.clientWidth-32 && box.height*zoom<=graph.clientHeight-32;};
    zoom=1;graphRender();
    if(!fits()){
      zoom=.25;graphRender();
      if(fits()){
        let fitting=.25,upper=1;
        for(let attempt=0;attempt<12;attempt++){zoom=(fitting+upper)/2;graphRender();if(fits())fitting=zoom;else upper=zoom;}
        zoom=fitting;graphRender();
      }
    }
    const bounds=occupied(overview);if(!bounds)return;
    restoreViewport({left:Math.max(0,(bounds.left+bounds.width/2)*zoom-graph.clientWidth/2),top:Math.max(0,(bounds.top+bounds.height/2)*zoom-graph.clientHeight/2)});framePending=false;
    const overflow=bounds.width*zoom>graph.clientWidth-31 || bounds.height*zoom>graph.clientHeight-31;
    find('graph-frame').hidden=false;find('graph-frame').textContent=overview?`Overview at ${Math.round(zoom*100)}%. ${overflow?'Content still overflows; pan or use the complete Table.':'Return to Fit selection for readable inspection.'}`:overflow?'Selection overflows; pan to its remaining declarations.':'Selection framed.';
  }
  function graphRender(){const owned=focusedDeclaration(),position=viewport();graph();summary();selection();restoreViewport(position);restoreDeclaration(owned);}
  function graph() {
    ++revealEpoch;
    typeButtons=typeButtons.filter(([element])=>!find('scene').contains(element));linkButtons=linkButtons.filter(([element])=>!find('scene').contains(element));
    drawn=renderOntologyGraph(find,node,typeButton,linkButton,cardinality,projection(),{type:selectedType,link:selectedLink},scope,zoom,relatedCount);
    ({width,height}=drawn);graphNodes=drawn.nodes;graphEdges=drawn.edges;graphLabels=drawn.labels;
    typeButtons=typeButtons.filter(([element])=>element.isConnected);linkButtons=linkButtons.filter(([element])=>element.isConnected);
    if(scope==='related' && drawn.plan)relatedCount=drawn.plan.neighbors;
    scale(zoom);
  }
  function table() {
    find('model-table').replaceChildren(
      ontologyTable('Object type declarations', ['Object type','Declaration revision','Properties'], schemas.map(row => [typeButton(row),row.revision,row.properties.length])),
      ontologyTable('Relationship declarations', ['Relationship','From','To','Cardinality','Declaration revision'], links.map(row => [linkButton(row),typeText(row.from_entity_type),typeText(row.to_entity_type),cardinality(row.cardinality),row.revision])),
    );
  }
  function viewport(){return {left:find('graph').scrollLeft,top:find('graph').scrollTop};}
  function restoreViewport(value){find('graph').scrollLeft=value.left;find('graph').scrollTop=value.top;cameraViewport=viewport();}
  function manualCamera(){cameraManual=true;framePending=false;}
  function scopeCounts() {
    const projected=projection(),count=(kind,displayed)=>({total:projected.counts[kind].total,eligible:projected[kind==='types'?'schemas':'links'].length,displayed,remaining:projected[kind==='types'?'schemas':'links'].length-displayed,outsideScope:projected.counts[kind].remaining,undrawn:projected.counts[kind].undrawn});
    return {types:count('types',graphNodes.length),relationships:count('relationships',graphEdges.length)};
  }
  function summary() {
    const counts=scopeCounts(),label=value=>`${value.displayed} of ${value.eligible} eligible shown, ${value.remaining} remaining, ${value.outsideScope} outside scope, ${value.undrawn} undrawn`;
    find('model-summary').textContent=`${schemas.length} type declarations · ${links.length} relationship declarations · ${counts.types.displayed} types and ${counts.relationships.displayed} relationships shown.`;
    find('graph-counts').textContent=`${scope} graph: ${label(counts.types)} types · ${label(counts.relationships)} relationships. Table retains every declaration.`;
    for(const value of ['selected','related','all'])find(`scope-${value}`).setAttribute('aria-pressed',String(scope===value));
    const expand=find('expand'),active=document.activeElement===expand;expand.disabled=scope!=='related' || !drawn?.plan || drawn.plan.neighbors>=drawn.plan.totalNeighbors;expand.hidden=expand.disabled;
    expand.textContent=`Expand next${drawn?.plan?' ('+(drawn.plan.totalNeighbors-drawn.plan.neighbors)+')':''}`;
    if(active && expand.disabled && document.activeElement===document.body)find('fit').focus({preventScroll:true});
  }
  function refreshGraph(resized=false) {
    const owned=focusedDeclaration(),position=viewport();graph();summary();selection();if(framePending && drawn.plan)frameSelection();else{restoreViewport(position);if(resized && !cameraManual && drawn.plan)frameSelection(true,owned);}restoreDeclaration(owned);
  }
  function setScope(value) {
    if(!context || !['selected','related','all'].includes(value))return false;
    if(scope!==value){const owned=focusedDeclaration();scope=value;zoom=1;framePending=true;refreshGraph();library(true);restoreDeclaration(owned);}return true;
  }
  function setModel(rows, relations, selected, modelSource = 'Published declarations', modelContext=null,catalog=null) {
    graphObserver?.observe(find('graph'));
    const key=ontologyModelSourceKey(modelContext),same=key!==null && key===context;
    const owned=focusedDeclaration(),position=same?(heldViewport || viewport()):{left:0,top:0};
    const retained=heldSelection || {type:selectedType,link:selectedLink};
    selected ??= same && (retained.type || retained.link)?retained:{type:rows[0]?.entity_type || null,link:null};
    if(!same){scope='related';zoom=1;relatedCount=null;framePending=true;for(const kind of ['type','link','value'])find(`library-${kind}-group`).open=kind==='type';}
    if(same && (selected?.type!==selectedType || selected?.link!==selectedLink) && !heldSelection){relatedCount=null;zoom=1;framePending=true;}context=key;heldViewport=null;heldSelection=null;
    schemas = rows; links = relations;values=catalog || [];catalogAvailable=catalog!==null;find('values-note').textContent=catalogAvailable?'Complete reusable catalog from this authorized model snapshot. Read-only; declarations are not changed here.':'Complete reusable catalog requires native model access. Object schemas contain used definitions only.'; source = modelSource; typeButtons = []; linkButtons = [];
    const projected=ontologyModelProjection(rows,relations,selected,scope);
    selectedLink=projected.linkById.has(selected?.link)?selected.link:null;
    selectedType=!selectedLink && projected.typeById.has(selected?.type)?selected.type:null;
    library(true);graph();table();
    if(selectedLink)selectLink(selectedLink);else select(selectedType);
    summary();if(framePending && drawn?.plan)frameSelection();else restoreViewport(position);restoreDeclaration(owned,same);return {type:selectedType,link:selectedLink};
  }
  function clear(retaining=false) {
    graphObserver?.disconnect();find('graph-frame').textContent='';find('graph-frame').hidden=true;find('graph-counts').textContent='';
    const active = document.activeElement;
    const restore = ['scene','library-types','library-links','model-table','link','selection-details'].some(id => find(id).contains(active));
    heldViewport=retaining?(heldViewport || viewport()):null;
    heldSelection=retaining?(heldSelection || {type:selectedType,link:selectedLink}):null;
    if(!retaining){context=null;scope='related';zoom=1;relatedCount=null;framePending=true;cameraManual=false;pointerScroll=false;}
    values=[];catalogAvailable=false;find('values-note').textContent='';schemas = []; links = []; drawn=null;typeButtons = []; linkButtons = []; graphLabels = []; graphNodes = []; graphEdges = []; selectedType = null; selectedLink = null;
    for (const id of ['scene','library-types','library-links','library-values','model-table','link-properties']) find(id).replaceChildren();
    width = 640; height = 360; find('scene').style.width = `${width}px`; find('scene').style.height = `${height}px`; scale(zoom);
    if(!retaining)restoreViewport({left:0,top:0});
    find('library-summary').textContent = '';find('library-scope').textContent='';for(const kind of ['type','link','value'])find(`library-${kind}-group`).open=kind==='type'; find('model-summary').textContent = ''; find('link').hidden = true; find('inspector-empty').hidden = false;
    renderOntologySelectedContext(null);
    if (restore) fallback();
  }
  function showMode(value) {
    const prior = value === 'schema' ? find('objects') : find('model');
    const restore = prior.contains(document.activeElement);
    find('model').hidden = value !== 'schema'; find('objects').hidden = value !== 'objects';
    for (const id of ['schema','objects']) find(`mode-${id}`).setAttribute('aria-pressed', String(value === id));
    modeChanged(value);
    if (restore) find(`mode-${value}`).focus({preventScroll:true});
  }
  function showRepresentation(value) {
    const old = representation === 'graph' ? find('graph') : find('model-table');
    const restore = old.contains(document.activeElement);
    representation = value;
    find('graph').hidden = value !== 'graph'; find('model-table').hidden = value !== 'table';
    find('graph-controls').hidden = value !== 'graph'; find('graph-help').hidden = value !== 'graph';
    for (const id of ['graph','table']) find(`view-${id}`).setAttribute('aria-pressed', String(value === id));
    if (restore) find(`view-${value}`).focus({preventScroll:true});
  }
  const scopeControls=node('div','','foundry-scope-controls');scopeControls.setAttribute('role','group');scopeControls.setAttribute('aria-label','Declaration scope');
  for(const value of ['selected','related','all']){const control=button(value[0].toUpperCase()+value.slice(1),()=>setScope(value));control.id=`live-ontology-scope-${value}`;scopeControls.append(control);}
  const expand=button('Expand next related type',()=>{if(scope==='related' && drawn?.plan){relatedCount=drawn.plan.neighbors+1;framePending=false;refreshGraph();}});expand.id='live-ontology-expand';
  const overview=button('Overview',()=>fit(true));overview.id='live-ontology-overview';
  const frame=node('output');frame.id='live-ontology-graph-frame';frame.hidden=true;frame.setAttribute('aria-live','polite');
  const counts=node('details','','foundry-graph-counts'),countsTitle=node('summary','Counts'),countsBody=node('p');countsBody.id='live-ontology-graph-counts';counts.append(countsTitle,countsBody);
  find('graph-controls').setAttribute('aria-label','Type graph controls');find('graph-controls').append(scopeControls,expand,counts,overview,frame);find('fit').textContent='Fit selection';
  find('graph-help').textContent='Arrows follow native from → to direction. Selected shows the accepted declaration; Related adds a measured first neighborhood. Expand deliberately; All and Table retain complete declarations. Pan with scroll or arrow keys. + and − zoom; Home frames selection. Overview may shrink text and still overflow.';
  const graphObserver=typeof ResizeObserver==='function'?new ResizeObserver(()=>{if(context && find('graph').getClientRects().length)refreshGraph(true);}):null;
  graphObserver?.observe(find('graph'));
  find('search').addEventListener('input', ()=>library());
  for (const value of ['schema','objects']) find(`mode-${value}`).addEventListener('click', () => showMode(value));
  for (const value of ['graph','table']) find(`view-${value}`).addEventListener('click', () => showRepresentation(value));
  find('zoom-in').addEventListener('click',()=>{manualCamera();scale(zoom+.25);graphRender();});
  find('zoom-out').addEventListener('click',()=>{manualCamera();scale(zoom-.25);graphRender();});
  find('fit').addEventListener('click',()=>fit());
  find('graph').addEventListener('wheel',manualCamera,{passive:true});
  find('graph').addEventListener('touchmove',manualCamera,{passive:true});
  find('graph').addEventListener('pointerdown',event=>{++revealEpoch;pointerScroll=event.target===find('graph');});
  const pointerPan=()=>{const position=viewport();if(pointerScroll && cameraViewport && (position.left!==cameraViewport.left || position.top!==cameraViewport.top))manualCamera();};
  find('graph').addEventListener('scroll',pointerPan);
  for(const event of ['pointerup','pointercancel'])window.addEventListener(event,()=>{pointerPan();pointerScroll=false;});
  document.addEventListener('keydown',event=>{
    const ticket=++revealEpoch,source=context,prior=document.activeElement;
    if(event.key!=='Tab' || !event.isTrusted || event.defaultPrevented || !source)return;
    requestAnimationFrame(()=>{
      const active=document.activeElement,graph=find('graph');
      if(ticket!==revealEpoch || event.defaultPrevented || source!==context || active===prior || !active?.isConnected || active.disabled || !graph.isConnected || !graph.getClientRects().length || !active.getClientRects().length || !graphNodes.includes(active) && !graphLabels.includes(active))return;
      const box=graph.getBoundingClientRect(),rect=active.getBoundingClientRect(),left=box.left+graph.clientLeft,top=box.top+graph.clientTop;
      const shift=(start,end,min,max)=>end-start>max-min?0:start<min?Math.floor(start-min):end>max?Math.ceil(end-max):0;
      const x=shift(rect.left,rect.right,left,left+graph.clientWidth),y=shift(rect.top,rect.bottom,top,top+graph.clientHeight);
      if(x || y)restoreViewport({left:graph.scrollLeft+x,top:graph.scrollTop+y});
    });
  });
  find('graph').addEventListener('keydown', event => {
    if (event.target !== find('graph')) return;
    const direction = {ArrowLeft:[-80,0],ArrowRight:[80,0],ArrowUp:[0,-80],ArrowDown:[0,80]}[event.key];
    if (direction) { manualCamera();event.preventDefault(); find('graph').scrollBy({left:direction[0],top:direction[1],behavior:'auto'}); }
    else if (['+','=','-','Home'].includes(event.key)) { event.preventDefault(); if (event.key === 'Home') fit(); else{manualCamera();scale(zoom + (event.key === '-' ? -0.25 : 0.25));graphRender();} }
    else if(['PageUp','PageDown','End',' '].includes(event.key))manualCamera();
  });
  return {setModel,select,selectLink,setScope,scope:()=>({mode:scope,counts:scopeCounts()}),clear,reset:()=>{clear();find('search').value='';zoom=1;showMode('schema');showRepresentation('graph');scale(1);}};
}
"#;
