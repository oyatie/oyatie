pub(super) const SCRIPT: &str = r#"
  function clearRelationshipResults() {
    const rows = find('relationship-results');
    const restoreFocus = rows.contains(document.activeElement);
    rows.replaceChildren(); find('relationship-status').textContent = '';
    if (restoreFocus) find('relationship-read').focus({preventScroll:true});
  }
  function clearRelationships() {
    const active = document.activeElement;
    const restoreFocus = find('relationship-results').contains(active) || ['relationship-type','relationship-direction','relationship-read'].some(id => find(id) === active);
    clearRelationshipResults();
    find('relationship-type').replaceChildren(); find('relationship-read').disabled = true;
    find('relationship-root').textContent = '';
    if (restoreFocus) focusSchemaControl();
  }
  function renderRelationshipChoices(reset = false) {
    clearRelationshipResults();
    if (!activeObject) { clearRelationships(); return; }
    if (reset) find('relationship-direction').value = 'outbound';
    const direction = find('relationship-direction').value;
    const eligible = links.filter(link => link[direction === 'outbound' ? 'from_entity_type' : 'to_entity_type'] === activeObject.entity_type);
    const placeholder = document.createElement('option'); placeholder.value = ''; placeholder.textContent = 'Choose a relationship';
    find('relationship-type').replaceChildren(placeholder, ...eligible.map(link => {
      const option = document.createElement('option'); option.value = link.link_type;
      const label = type => schemas.find(schema => schema.entity_type === type)?.display_name || type;
      option.textContent = `${link.link_type} · ${label(link.from_entity_type)} → ${label(link.to_entity_type)}`;
      return option;
    }));
    find('relationship-root').textContent = `Root ${activeObject.object_ref} · selected type ${activeObject.entity_type} · requested schema ${activeObject.revision}. Read one selected relationship in one direction.`;
    find('relationship-read').disabled = true;
    find('relationship-status').textContent = eligible.length ? 'Choose an accepted relationship, then Read related objects.' : `No accepted ${direction} relationships for this object type. Choose another direction to check its declarations.`;
  }
  async function readRelationships() {
    if (!activeObject) return;
    const link = links.find(link => link.link_type === find('relationship-type').value);
    const direction = find('relationship-direction').value;
    if (!link || !['outbound','inbound'].includes(direction) || link[direction === 'outbound' ? 'from_entity_type' : 'to_entity_type'] !== activeObject.entity_type) return;
    const request = {idempotency_key:crypto.randomUUID(),root:{...activeObject},link_type:link.link_type,direction,observed_at_epoch_seconds:Math.floor(Date.now()/1000)};
    const frozen = JSON.stringify(request);
    clearRelationshipResults();
    return run('Reading authorized related objects…', async token => {
      const graph = await fetchRead('search-around',token,frozen); if (!graph || !valid(token)) return;
      const nodes = decodeOntologyRelationships(graph,request,link);
      const neighbors = graph.edges.map(edge => ({edge,node:nodes.get(direction === 'outbound' ? edge.to_entity_id : edge.from_entity_id)}));
      find('relationship-results').replaceChildren(ontologyTable('Related native objects · one relationship, one hop', ['Relationship','From object','To object','Related object','Entity type','Stored schema revision'], neighbors.map(({edge,node}) => {
        const button = document.createElement('button'); button.type='button'; button.textContent=node.entity_id + (node.entity_id === request.root.object_ref ? ' (self relationship)' : ''); button.dataset.liveObject=node.entity_id;
        button.addEventListener('click',()=>detail({entity_type:node.entity_type_id,object_ref:node.entity_id,revision:node.written_revision},button));
        return [edge.edge_type_id,edge.from_entity_id,edge.to_entity_id,button,node.entity_type_id,node.written_revision];
      })));
      const status = graph.truncated ? 'INCOMPLETE: the native query reached its result bound. This is a partial one-hop result; no next page is available here.'
        : neighbors.length ? `${neighbors.length} related native objects from the selected relationship.` : 'No related objects for this relationship and direction. The root exists and this read completed.';
      find('relationship-status').textContent = status + ' Request time is caller intent metadata, not a current observation timestamp.';
      note(graph.truncated ? 'Related object read is INCOMPLETE. See relationship results.' : 'Related object read completed. Open an object at its own stored schema revision.');
    });
  }
  find('relationship-direction').addEventListener('input',()=>{invalidate();busy(false);renderRelationshipChoices();});
  find('relationship-type').addEventListener('input',()=>{invalidate();busy(false);clearRelationshipResults();find('relationship-read').disabled=!find('relationship-type').value;});
  find('relationship-read').addEventListener('click',readRelationships);
"#;
