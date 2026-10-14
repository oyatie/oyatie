pub(super) const SCRIPT: &str = r#"
function revealOntologyDetails(element, root) {
  if(!element?.isConnected || !root.contains(element) || element.closest('[hidden]'))return false;
  for(let owner=element;owner && owner!==root;owner=owner.parentElement)if(owner.tagName.toLowerCase()==='details')owner.open=true;
  return true;
}
function createOntologyModelView(selectType, fallback, selectRelationship = () => {}, modeChanged = () => {}) {
  const find = id => document.getElementById(`live-ontology-${id}`);
  let schemas = [], links = [], selectedType = null, selectedLink = null;
  let representation = 'graph', zoom = 1, width = 640, height = 360;
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
  function typeButton(schema, className = '') {
    const element = button('', () => selectType(schema.entity_type), className);
    element.append(node('span', schema.display_name), node('small', ` · ${schema.entity_type}`));
    element.dataset.modelType = schema.entity_type; typeButtons.push([element, schema.entity_type]); return element;
  }
  function linkButton(link, className = '') {
    const element = button(link.link_type, () => selectLink(link.link_type), className);
    element.dataset.modelLink = link.link_type; linkButtons.push([element, link.link_type]); return element;
  }
  function selection() {
    for (const [element, id] of typeButtons) element.setAttribute('aria-pressed', String(!selectedLink && selectedType === id));
    for (const [element, id] of linkButtons) element.setAttribute('aria-pressed', String(selectedLink === id));
  }
  function selectLink(id) {
    const link = links.find(row => row.link_type === id); if (!link) return;
    const active = document.activeElement, restore = find('schema').contains(active);
    selectedLink = id; find('schema').hidden = true;
    find('link-properties').replaceChildren(ontologyTable('Relationship declaration', ['Declaration', 'Value'], [
      ['Relationship type', link.link_type], ['From entity type (source)', typeText(link.from_entity_type)], ['To entity type (target)', typeText(link.to_entity_type)],
      ['Cardinality', `${cardinality(link.cardinality)} (${link.cardinality})`], ['Limits per relationship type and tenant', cardinalityLimits(link.cardinality)],
      ['Cross-tenant intent', String(link.allow_cross_tenant)], ['Declaration revision', link.revision],
    ]));
    find('link').hidden = false; find('inspector-empty').hidden = true;find('inspector-context').textContent=`Relationship · ${link.link_type} · ${typeText(link.from_entity_type)} → ${typeText(link.to_entity_type)}`; selection(); selectRelationship(id);
    if (restore) fallback();
  }
  function typeText(id) { return `${schemas.find(row => row.entity_type === id)?.display_name || id} · ${id}`; }
  function select(id) {
    const restore = find('link').contains(document.activeElement);
    selectedType = id; selectedLink = null; find('link-properties').replaceChildren(); find('link').hidden = true;
    find('inspector-context').textContent=schemas.some(row=>row.entity_type===id)?`Type · ${typeText(id)}`:'';
    find('inspector-empty').hidden = Boolean(id); selection(); if (restore) fallback();
  }
  function library() {
    const active = document.activeElement;
    const restore = find('library-types').contains(active) || find('library-links').contains(active);
    const search = find('search').value.toLocaleLowerCase();
    const types = schemas.filter(row => [row.display_name, row.entity_type, ...row.properties.flatMap(p => [p.name, ontologyDeclarationText(p.value_type)])].some(value => value.toLocaleLowerCase().includes(search)));
    const relations = links.filter(row => [row.link_type,typeText(row.from_entity_type),typeText(row.to_entity_type)].some(value => value.toLocaleLowerCase().includes(search)));
    typeButtons = typeButtons.filter(([element]) => !find('library-types').contains(element));
    linkButtons = linkButtons.filter(([element]) => !find('library-links').contains(element));
    find('library-types').replaceChildren(...(types.length ? types.map(row => typeButton(row, 'foundry-library-item')) : [node('p', schemas.length ? 'No matching types.' : 'No type declarations.')]));
    find('library-links').replaceChildren(...(relations.length ? relations.map(row => linkButton(row, 'foundry-library-item')) : [node('p', links.length ? 'No matching relationships.' : 'No relationship declarations.')]));
    find('library-summary').textContent = `${types.length} of ${schemas.length} types · ${relations.length} of ${links.length} relationships`;
    selection(); if (restore) find('search').focus({preventScroll:true});
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
  function fit() {
    const graph = find('graph');
    const availableWidth = graph.clientWidth - 32, availableHeight = graph.clientHeight - 32;
    let value = 1;
    function limit(right, bottom, fixedWidth = 0, fixedHeight = 0) {
      if (right > 0) value = Math.min(value, (availableWidth - fixedWidth) / right);
      if (bottom > 0) value = Math.min(value, (availableHeight - fixedHeight) / bottom);
    }
    for (const card of graphNodes) limit(card.offsetLeft + card.offsetWidth, card.offsetTop + card.offsetHeight);
    for (const edge of graphEdges) {
      const box = edge.getBBox(), padding = Math.hypot(arrowRefX, arrowRefY) * Number.parseFloat(getComputedStyle(edge).strokeWidth);
      limit(box.x + box.width + padding, box.y + box.height + padding);
    }
    // Inverse-scaled labels keep their screen dimensions when Fit changes the scene zoom.
    for (const label of graphLabels) limit(label.offsetLeft, label.offsetTop, label.offsetWidth, label.offsetHeight);
    if (!graphNodes.length) limit(width, height);
    scale(value);
    graph.scrollLeft = 0; graph.scrollTop = 0;
  }
  function graph() {
    const scene = find('scene'); scene.replaceChildren(); graphLabels = []; graphNodes = []; graphEdges = [];
    const columns = Math.min(3, Math.max(1, Math.ceil(Math.sqrt(schemas.length))));
    width = Math.max(640, columns * 350 + 100); height = Math.max(360, Math.ceil(schemas.length / columns) * 260 + 100);
    scene.style.width = `${width}px`; scene.style.height = `${height}px`;
    const positions = new Map(schemas.map((row, index) => [row.entity_type, {x: 70 + index % columns * 350, y: 60 + Math.floor(index / columns) * 260}]));
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('width', width); svg.setAttribute('height', height); svg.setAttribute('aria-hidden', 'true'); svg.classList.add('foundry-edges');
    const defs = document.createElementNS(svg.namespaceURI, 'defs'), marker = document.createElementNS(svg.namespaceURI, 'marker');
    marker.setAttribute('id', 'foundry-native-arrow'); marker.setAttribute('markerWidth',String(arrowSize)); marker.setAttribute('markerHeight',String(arrowSize)); marker.setAttribute('refX',String(arrowRefX)); marker.setAttribute('refY',String(arrowRefY)); marker.setAttribute('orient','auto');
    const arrow = document.createElementNS(svg.namespaceURI, 'path'); arrow.setAttribute('d','M 0 0 L 8 4 L 0 8 z'); marker.append(arrow); defs.append(marker); svg.append(defs); scene.append(svg);
    links.forEach((link, index) => {
      const from = positions.get(link.from_entity_type), to = positions.get(link.to_entity_type);
      if (!from || !to) return;
      const self = link.from_entity_type === link.to_entity_type;
      const edge = document.createElementNS(svg.namespaceURI, 'path');
      const startX = from.x + 130, startY = from.y + 200, endX = to.x + 130, endY = to.y;
      edge.setAttribute('d', self ? `M ${from.x + 260} ${from.y + 40} C ${from.x + 335} ${from.y - 10}, ${from.x + 335} ${from.y + 140}, ${from.x + 260} ${from.y + 115}` : `M ${startX} ${startY} C ${startX} ${startY + 40}, ${endX} ${endY - 40}, ${endX} ${endY}`);
      edge.setAttribute('marker-end', 'url(#foundry-native-arrow)'); edge.dataset.modelEdge = link.link_type; svg.append(edge); graphEdges.push(edge);
      const label = linkButton(link, 'foundry-edge-label'); graphLabels.push(label);
      label.append(node('small', cardinality(link.cardinality)));
      label.style.left = `${self ? from.x + 265 : (startX + endX) / 2 - 75}px`;
      label.style.top = `${self ? from.y + 140 : (startY + endY) / 2 + (index % 3) * 22}px`;
      label.setAttribute('aria-label', `Inspect relationship ${link.link_type}, ${typeText(link.from_entity_type)} to ${typeText(link.to_entity_type)}, ${cardinality(link.cardinality)}`); scene.append(label);
    });
    for (const schema of schemas) {
      const position = positions.get(schema.entity_type), card = typeButton(schema, 'foundry-type-node');
      card.textContent = ''; card.append(node('span', 'OBJECT TYPE', 'foundry-node-eyebrow'), node('strong', schema.display_name), node('small', `${schema.entity_type} · Declaration revision ${schema.revision}`));
      for (const property of schema.properties.slice(0,3)) {
        const row = node('span', '', 'foundry-node-property'); row.append(node('span',property.name),node('small',ontologyDeclarationText(property.value_type)));
        if (property.name === schema.primary_key_property) row.append(node('em','Primary'));
        if (property.name === schema.title_property) row.append(node('em','Title'));
        card.append(row);
      }
      card.append(node('small',`${schema.properties.length} declared properties · ${schema.properties.filter(p=>p.required).length} required`));
      card.style.left = `${position.x}px`; card.style.top = `${position.y}px`; card.setAttribute('aria-label', `Inspect entity type ${schema.display_name}, ${schema.entity_type}, declaration revision ${schema.revision}`); scene.append(card); graphNodes.push(card);
    }
    if (!schemas.length) scene.append(node('p','No native type declarations.','foundry-graph-empty'));
    scale(zoom);
  }
  function table() {
    find('model-table').replaceChildren(
      ontologyTable('Object type declarations', ['Object type','Declaration revision','Properties'], schemas.map(row => [typeButton(row),row.revision,row.properties.length])),
      ontologyTable('Relationship declarations', ['Relationship','From','To','Cardinality','Declaration revision'], links.map(row => [linkButton(row),typeText(row.from_entity_type),typeText(row.to_entity_type),cardinality(row.cardinality),row.revision])),
    );
  }
  function setModel(rows, relations, selected) {
    schemas = rows; links = relations; typeButtons = []; linkButtons = [];
    selectedType = selected; selectedLink = null;
    library(); graph(); table(); select(selected);
    find('model-summary').textContent = `${schemas.length} type declarations · ${links.length} relationship declarations`;
  }
  function clear() {
    const active = document.activeElement;
    const restore = ['scene','library-types','library-links','model-table','link'].some(id => find(id).contains(active));
    schemas = []; links = []; typeButtons = []; linkButtons = []; graphLabels = []; graphNodes = []; graphEdges = []; selectedType = null; selectedLink = null;
    for (const id of ['scene','library-types','library-links','model-table','link-properties']) find(id).replaceChildren();
    width = 640; height = 360; find('scene').style.width = `${width}px`; find('scene').style.height = `${height}px`; scale(zoom);
    find('library-summary').textContent = ''; find('model-summary').textContent = ''; find('link').hidden = true; find('inspector-empty').hidden = false;
    find('inspector-context').textContent='';
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
  find('search').addEventListener('input', library);
  for (const value of ['schema','objects']) find(`mode-${value}`).addEventListener('click', () => showMode(value));
  for (const value of ['graph','table']) find(`view-${value}`).addEventListener('click', () => showRepresentation(value));
  find('zoom-in').addEventListener('click', () => scale(zoom + 0.25));
  find('zoom-out').addEventListener('click', () => scale(zoom - 0.25));
  find('fit').addEventListener('click', fit);
  find('graph').addEventListener('keydown', event => {
    if (event.target !== find('graph')) return;
    const direction = {ArrowLeft:[-80,0],ArrowRight:[80,0],ArrowUp:[0,-80],ArrowDown:[0,80]}[event.key];
    if (direction) { event.preventDefault(); find('graph').scrollBy({left:direction[0],top:direction[1],behavior:'auto'}); }
    else if (['+','=','-','Home'].includes(event.key)) { event.preventDefault(); if (event.key === 'Home') fit(); else scale(zoom + (event.key === '-' ? -0.25 : 0.25)); }
  });
  return {setModel,select,selectLink,clear,reset:()=>{clear();find('search').value='';zoom=1;showMode('schema');showRepresentation('graph');scale(1);}};
}
"#;
