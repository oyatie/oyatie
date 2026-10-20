pub(super) const SCRIPT: &str = r#"
function renderOntologyGraph(find, node, typeButton, linkButton, cardinality, projected, selected, scope, zoom, retained) {
  const graph=find('graph'),scene=find('scene');scene.replaceChildren();
  graph.style.setProperty('--foundry-graph-width',`${graph.clientWidth}px`);
  const cards=new Map(),labels=new Map(),sizes={types:new Map(),links:new Map()};
  for(const row of projected.schemas){
    const card=typeButton(row,'foundry-type-node');card.textContent='';
    card.style.minHeight=`max(${44/zoom}px,5.25rem)`;card.style.minWidth=`${44/zoom}px`;
    card.append(node('strong',row.display_name),node('small',`${row.properties.length} properties`));
    card.setAttribute('aria-label',`Inspect entity type ${row.display_name}, ${row.entity_type}, declaration revision ${row.revision}`);
    card.setAttribute('title',`${row.display_name} · ${row.entity_type} · Declaration revision ${row.revision}`);
    scene.append(card);cards.set(row.entity_type,card);sizes.types.set(row.entity_type,{w:card.offsetWidth,h:card.offsetHeight});
  }
  for(const row of projected.links){
    const label=linkButton(row,'foundry-edge-label');label.textContent='';
    const text=node('span',row.display?.display_name || row.link_type,'foundry-edge-text');
    text.append(node('small',cardinality(row.cardinality)));label.append(text);
    label.style.transform=`scale(${1/zoom})`;
    label.setAttribute('aria-label',`Inspect relationship ${row.link_type}, ${row.from_entity_type} to ${row.to_entity_type}, ${cardinality(row.cardinality)}`);
    scene.append(label);labels.set(row.link_type,label);sizes.links.set(row.link_type,{w:label.offsetWidth/zoom,h:label.offsetHeight/zoom});
  }
  const padding=Math.max(12,Number.parseFloat(getComputedStyle(graph).fontSize));
  const plan=ontologyGraphPlan(projected.schemas,projected.links,selected,sizes,{width:graph.clientWidth,height:graph.clientHeight,padding},scope,retained);
  if(!plan){scene.replaceChildren(node('p','Graph measurements are unavailable. Use the complete Table.','foundry-graph-empty'));return {width:1,height:1,nodes:[],labels:[],edges:[],plan:null};}
  const shownTypes=new Set(plan.nodes.map(row=>row.id)),shownLinks=new Set(plan.edges.map(row=>row.id));
  for(const[id,card]of cards)if(!shownTypes.has(id))scene.removeChild(card);
  for(const[id,label]of labels)if(!shownLinks.has(id))scene.removeChild(label);
  const svg=document.createElementNS('http://www.w3.org/2000/svg','svg');svg.setAttribute('width',plan.width);svg.setAttribute('height',plan.height);svg.setAttribute('aria-hidden','true');svg.classList.add('foundry-edges');
  const defs=document.createElementNS(svg.namespaceURI,'defs');svg.append(defs);scene.append(svg);
  const edges=[],routes=new Map(),reading={focus:null,hover:null,current:null};
  function cue(){
    const id=reading.focus ?? reading.hover ?? (routes.has(selected.link)?selected.link:null);
    if(id===reading.current)return;
    for(const[key,active]of [[reading.current,false],[id,true]])for(const part of routes.get(key) || [])part.setAttribute('data-model-reading',String(active));
    if(id!==null)svg.append(routes.get(id)[0]);
    reading.current=id;
  }
  for(const row of plan.edges){
    const marker=document.createElementNS(svg.namespaceURI,'marker'),arrow=document.createElementNS(svg.namespaceURI,'path'),markerId='foundry-native-arrow-'+edges.length;
    for(const[key,value]of Object.entries({id:markerId,markerWidth:8,markerHeight:8,refX:7,refY:4,orient:'auto'}))marker.setAttribute(key,String(value));
    arrow.setAttribute('d','M 0 0 L 8 4 L 0 8 z');arrow.dataset.modelArrow=row.id;marker.append(arrow);defs.append(marker);
    const path=document.createElementNS(svg.namespaceURI,'path');path.setAttribute('d',row.path);path.setAttribute('marker-end','url(#'+markerId+')');path.dataset.modelEdge=row.id;svg.append(path);edges.push(path);routes.set(row.id,[path,arrow]);
    const label=labels.get(row.id);label.style.left=`${row.label.x}px`;label.style.top=`${row.label.y}px`;
    const track=(kind,active)=>{
      if(!label.isConnected)return;
      if(active)reading[kind]=row.id;else if(reading[kind]===row.id)reading[kind]=null;
      cue();
    };
    for(const[event,kind,active]of [['focus','focus',true],['blur','focus',false],['mouseenter','hover',true],['mouseleave','hover',false]])label.addEventListener(event,()=>track(kind,active));
  }
  cue();
  for(const row of plan.nodes){const card=cards.get(row.id);card.style.left=`${row.x}px`;card.style.top=`${row.y}px`;}
  scene.style.width=`${plan.width}px`;scene.style.height=`${plan.height}px`;
  if(!plan.nodes.length)scene.append(node('p','No unambiguous types in this scope.','foundry-graph-empty'));
  return {width:plan.width,height:plan.height,nodes:plan.nodes.map(row=>cards.get(row.id)),labels:plan.edges.map(row=>labels.get(row.id)),edges,plan};
}
"#;
