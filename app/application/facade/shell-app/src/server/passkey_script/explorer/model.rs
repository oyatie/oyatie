pub(super) const SCRIPT: &str = r#"
function revealOntologyDetails(element, root) {
  if(!element?.isConnected || !root.contains(element) || element.closest('[hidden]'))return false;
  for(let owner=element;owner && owner!==root;owner=owner.parentElement)if(owner.tagName.toLowerCase()==='details')owner.open=true;
  return true;
}
function createOntologyModelView(selectType, fallback, selectRelationship = () => {}, modeChanged = () => {}, selectValueType = () => false) {
  const find = id => document.getElementById(`live-ontology-${id}`);
  let values=[],valueButtons=[],selectedValue=null;
  let schemas = [], links = [], selectedType = null, selectedLink = null, source = 'Published declarations';
  let representation = 'graph', zoom = 1, width = 640, height = 360;
  let scope='related',context=null,heldViewport=null,heldSelection=null,relatedCount=null,drawn=null,framePending=true;
  const projection=()=>ontologyModelProjection(schemas,links,{type:selectedType,link:selectedLink,value:selectedValue},scope);
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
    return surface && (element?.dataset.modelType || element?.dataset.modelLink || element?.dataset.modelValue)?{element,surface,type:element.dataset.modelType,link:element.dataset.modelLink,value:element.dataset.modelValue}:null;
  }
  function restoreDeclaration(saved, same=true) {
    if(!saved || document.activeElement!==document.body && document.activeElement!==saved.element || saved.element.isConnected)return;
    const buttons=saved.value!==undefined?valueButtons:saved.type?typeButtons:linkButtons;
    const replacement=same?buttons.find(([element,id])=>id===(saved.value ?? saved.type ?? saved.link) && find(saved.surface).contains(element) && !element.disabled && element.getClientRects().length)?.[0]:null;
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
    return element;
  }
  const typeButton=(schema,className='')=>declarationButton(schema,true,className);
  const linkButton=(link,className='')=>declarationButton(link,false,className);
  function valueButton(row,index,className=''){
    const duplicate=values.filter(other=>other.id===row.id && other.revision===row.revision).length>1;
    const element=button(`${row.display_name} · ${row.id} · revision ${row.revision}${duplicate?' · duplicate row '+(index+1):''}`,()=>{
      if(element.disabled || !element.isConnected || !find('explorer').contains(element))return;
      const owned=focusedDeclaration(),prior=context;if(selectValueType(index)!==true || prior!==context)return;
      selectValue(index);restoreDeclaration(owned);
    },className);element.dataset.modelValue=String(index);valueButtons.push([element,String(index)]);return element;
  }
  function selectValue(index){
    if(!Number.isInteger(index) || !values[index])return false;selectedValue=index;
    find('schema').hidden=true;find('link').hidden=true;find('inspector-empty').hidden=true;
    renderOntologyValueDefinition(values[index],source);selection();return true;
  }
  function selection() {
    for(const [element,index] of valueButtons)element.setAttribute('aria-pressed',String(selectedValue===Number(index)));
    for (const [element, id] of typeButtons) element.setAttribute('aria-pressed', String(selectedValue===null && !selectedLink && selectedType === id));
    for (const [element, id] of linkButtons) element.setAttribute('aria-pressed', String(selectedValue===null && selectedLink === id));
  }
  function selectLink(id) {
    selectedValue=null;renderOntologyValueDefinition(null,source);
    const link = projection().linkById.get(id); if (!link) return false;
    const changed=selectedLink!==id;if(changed){relatedCount=null;zoom=1;framePending=true;}
    const active = document.activeElement, restore = find('schema').contains(active);
    selectedLink = id; selectedType=null;if(changed)refreshGraph();find('schema').hidden = true;
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
    selectedValue=null;renderOntologyValueDefinition(null,source);
    if(id!==null && !projection().typeById.has(id))return false;
    const changed=selectedType!==id || selectedLink!==null;if(changed){relatedCount=null;zoom=1;framePending=true;}
    const restore = find('link').contains(document.activeElement);
    selectedType = id; selectedLink = null;if(changed)refreshGraph(); find('link-properties').replaceChildren(); find('link').hidden = true;
    renderOntologySelectedContext(schemas.some(row=>row.entity_type===id)?{source,kind:'type',type:typeContext(id)}:null);
    find('inspector-empty').hidden = Boolean(id); selection(); if (restore) fallback();return true;
  }
  function library(retaining=false) {
    const active = document.activeElement;
    const restore = ['library-types','library-links','library-values'].some(id=>find(id).contains(active));
    const search = find('search').value.toLocaleLowerCase();
    const types = schemas.filter(row => [row.display_name, row.entity_type, ...row.properties.flatMap(p => [p.name, ontologyDeclarationText(p.value_type)])].some(value => value.toLocaleLowerCase().includes(search)));
    const relations = links.filter(row => [row.link_type,typeText(row.from_entity_type),typeText(row.to_entity_type)].some(value => value.toLocaleLowerCase().includes(search)));
    typeButtons = typeButtons.filter(([element]) => !find('library-types').contains(element));
    linkButtons = linkButtons.filter(([element]) => !find('library-links').contains(element));
    find('library-types').replaceChildren(...(types.length ? types.map(row => typeButton(row, 'foundry-library-item')) : [node('p', schemas.length ? 'No matching types.' : 'No type declarations.')]));
    find('library-links').replaceChildren(...(relations.length ? relations.map(row => linkButton(row, 'foundry-library-item')) : [node('p', links.length ? 'No matching relationships.' : 'No relationship declarations.')]));
    valueButtons=valueButtons.filter(([element])=>!find('library-values').contains(element));
    const filtered=values.map((row,index)=>({row,index})).filter(({row})=>[row.id,row.display_name,String(row.revision)].some(value=>value.toLocaleLowerCase().includes(search)));
    find('library-values').replaceChildren(...filtered.map(({row,index})=>valueButton(row,index,'foundry-library-item')));
    find('library-summary').textContent = `${types.length} of ${schemas.length} types · ${relations.length} of ${links.length} relationships · ${filtered.length} of ${values.length} reusable value types`;
    selection(); if (restore && !retaining) find('search').focus({preventScroll:true});
  }
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
  function frameSelection() {
    const graph=find('graph'),bounds=occupied();if(!bounds)return;
    const first=graphNodes.find(card=>card.dataset.modelType===(selectedType || projection().linkById.get(selectedLink)?.from_entity_type));
    const target=bounds.width*zoom<=graph.clientWidth-32 && bounds.height*zoom<=graph.clientHeight-32?bounds:first?{left:first.offsetLeft,top:first.offsetTop,width:first.offsetWidth,height:first.offsetHeight}:bounds;
    graph.scrollLeft=Math.max(0,(target.left+target.width/2)*zoom-graph.clientWidth/2);graph.scrollTop=Math.max(0,(target.top+target.height/2)*zoom-graph.clientHeight/2);framePending=false;
  }
  function fit(overview=false) {
    const graph=find('graph');if(!graphNodes.length)return;
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
    graph.scrollLeft=Math.max(0,(bounds.left+bounds.width/2)*zoom-graph.clientWidth/2);graph.scrollTop=Math.max(0,(bounds.top+bounds.height/2)*zoom-graph.clientHeight/2);framePending=false;
    const overflow=bounds.width*zoom>graph.clientWidth-31 || bounds.height*zoom>graph.clientHeight-31;
    find('graph-frame').hidden=false;find('graph-frame').textContent=overview?`Overview at ${Math.round(zoom*100)}%. ${overflow?'Content still overflows; pan or use the complete Table.':'Return to Fit selection for readable inspection.'}`:overflow?'Selection overflows; pan to its remaining declarations.':'Selection framed.';
  }
  function graphRender(){const owned=focusedDeclaration(),position=viewport();graph();summary();selection();restoreViewport(position);restoreDeclaration(owned);}
  function graph() {
    typeButtons=typeButtons.filter(([element])=>!find('scene').contains(element));linkButtons=linkButtons.filter(([element])=>!find('scene').contains(element));
    drawn=renderOntologyGraph(find,node,typeButton,linkButton,cardinality,projection(),{type:selectedType,link:selectedLink,value:selectedValue},scope,zoom,relatedCount);
    ({width,height}=drawn);graphNodes=drawn.nodes;graphEdges=drawn.edges;graphLabels=drawn.labels;
    typeButtons=typeButtons.filter(([element])=>element.isConnected);linkButtons=linkButtons.filter(([element])=>element.isConnected);
    if(scope==='related' && drawn.plan)relatedCount=drawn.plan.neighbors;
    scale(zoom);
  }
  function table() {
    find('model-table').replaceChildren(
      ontologyTable('Object type declarations', ['Object type','Declaration revision','Properties'], schemas.map(row => [typeButton(row),row.revision,row.properties.length])),
      ontologyTable('Reusable value types',['Value type','Members'],values.map((row,index)=>[valueButton(row,index),row.members.length])),
      ontologyTable('Relationship declarations', ['Relationship','From','To','Cardinality','Declaration revision'], links.map(row => [linkButton(row),typeText(row.from_entity_type),typeText(row.to_entity_type),cardinality(row.cardinality),row.revision])),
    );
  }
  function viewport(){return {left:find('graph').scrollLeft,top:find('graph').scrollTop};}
  function restoreViewport(value){find('graph').scrollLeft=value.left;find('graph').scrollTop=value.top;}
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
  function refreshGraph() {
    const owned=focusedDeclaration(),position=viewport();graph();summary();selection();if(framePending && drawn.plan)frameSelection();else restoreViewport(position);restoreDeclaration(owned);
  }
  function setScope(value) {
    if(!context || !['selected','related','all'].includes(value))return false;
    if(scope!==value){scope=value;zoom=1;framePending=true;refreshGraph();}return true;
  }
  function setModel(rows, relations, selected, modelSource = 'Published declarations', modelContext=null,catalog=null) {
    graphObserver?.observe(find('graph'));
    const key=ontologyModelSourceKey(modelContext),same=key!==null && key===context;
    const owned=focusedDeclaration(),position=same?(heldViewport || viewport()):{left:0,top:0};
    const retained=heldSelection || {type:selectedType,link:selectedLink,value:selectedValue};
    selected ??= same && (retained.type || retained.link)?retained:{type:rows[0]?.entity_type || null,link:null};
    const valueSelection=selected && Object.hasOwn(selected,'value')?selected.value:same?(heldSelection?.value ?? selectedValue):null;
    if(valueSelection!==null)selected={type:selectedType,link:selectedLink,value:valueSelection};
    if(!same){scope='related';zoom=1;relatedCount=null;framePending=true;}
    if(same && (selected?.type!==selectedType || selected?.link!==selectedLink) && !heldSelection){relatedCount=null;zoom=1;framePending=true;}context=key;heldViewport=null;heldSelection=null;
    schemas = rows; links = relations;values=catalog || [];valueButtons=[];selectedValue=null;find('values-note').textContent=catalog===null?'Complete reusable catalog requires native model access. Object schemas contain used definitions only.':'Complete reusable catalog from this authorized model snapshot.';source = modelSource; typeButtons = []; linkButtons = [];
    const projected=ontologyModelProjection(rows,relations,selected,scope);
    selectedLink=projected.linkById.has(selected?.link)?selected.link:null;
    selectedType=!selectedLink && projected.typeById.has(selected?.type)?selected.type:null;
    library(true);graph();table();
    if(selectedLink)selectLink(selectedLink);else select(selectedType);
    if(valueSelection!==null && values[valueSelection])selectValue(valueSelection);
    summary();if(framePending && drawn?.plan)frameSelection();else restoreViewport(position);restoreDeclaration(owned,same);return selectedValue!==null?{type:null,link:null,value:selectedValue}:{type:selectedType,link:selectedLink,value:null};
  }
  function clear(retaining=false) {
    graphObserver?.disconnect();find('graph-frame').textContent='';find('graph-frame').hidden=true;find('graph-counts').textContent='';
    const active = document.activeElement;
    const restore = ['scene','library-types','library-links','library-values','model-table','link','value','selection-details'].some(id => find(id).contains(active));
    heldViewport=retaining?(heldViewport || viewport()):null;
    heldSelection=retaining?(heldSelection || {type:selectedType,link:selectedLink,value:selectedValue}):null;
    if(!retaining){context=null;scope='related';zoom=1;relatedCount=null;framePending=true;}
    values=[];valueButtons=[];selectedValue=null;renderOntologyValueDefinition(null,source);find('values-note').textContent='';schemas = []; links = []; drawn=null;typeButtons = []; linkButtons = []; graphLabels = []; graphNodes = []; graphEdges = []; selectedType = null; selectedLink = null;
    for (const id of ['scene','library-types','library-links','library-values','model-table','link-properties']) find(id).replaceChildren();
    width = 640; height = 360; find('scene').style.width = `${width}px`; find('scene').style.height = `${height}px`; scale(zoom);
    if(!retaining)restoreViewport({left:0,top:0});
    valueButtons=valueButtons.filter(([element])=>!find('library-values').contains(element));
    const filtered=values.map((row,index)=>({row,index})).filter(({row})=>[row.id,row.display_name,String(row.revision)].some(value=>value.toLocaleLowerCase().includes(search)));
    find('library-values').replaceChildren(...filtered.map(({row,index})=>valueButton(row,index,'foundry-library-item')));
    find('library-summary').textContent = ''; find('model-summary').textContent = ''; find('link').hidden = true; find('inspector-empty').hidden = false;
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
  const graphObserver=typeof ResizeObserver==='function'?new ResizeObserver(()=>{if(context && find('graph').getClientRects().length)refreshGraph();}):null;
  graphObserver?.observe(find('graph'));
  find('search').addEventListener('input', ()=>library());
  for (const value of ['schema','objects']) find(`mode-${value}`).addEventListener('click', () => showMode(value));
  for (const value of ['graph','table']) find(`view-${value}`).addEventListener('click', () => showRepresentation(value));
  find('zoom-in').addEventListener('click',()=>{framePending=false;scale(zoom+.25);graphRender();});
  find('zoom-out').addEventListener('click',()=>{framePending=false;scale(zoom-.25);graphRender();});
  find('fit').addEventListener('click',()=>fit());
  find('graph').addEventListener('keydown', event => {
    if (event.target !== find('graph')) return;
    const direction = {ArrowLeft:[-80,0],ArrowRight:[80,0],ArrowUp:[0,-80],ArrowDown:[0,80]}[event.key];
    if (direction) { framePending=false;event.preventDefault(); find('graph').scrollBy({left:direction[0],top:direction[1],behavior:'auto'}); }
    else if (['+','=','-','Home'].includes(event.key)) { event.preventDefault(); if (event.key === 'Home') fit(); else{framePending=false;scale(zoom + (event.key === '-' ? -0.25 : 0.25));graphRender();} }
  });
  return {setModel,select,selectLink,selectValue,setScope,scope:()=>({mode:scope,counts:scopeCounts()}),clear,reset:()=>{clear();find('search').value='';zoom=1;showMode('schema');showRepresentation('graph');scale(1);}};
}
"#;
