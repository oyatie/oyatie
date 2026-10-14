const CHECK: &str = r##"
(async () => {
  const workspace = input.html.indexOf('id="foundry-workspace"');
  const island = input.html.indexOf('id="dashboard-island-root"');
  assert.ok(workspace > input.html.indexOf('id="oyatie-sign-in"') && workspace < island, 'workspace has a stable shell position before the sample island');
  assert.ok(input.html.includes('</section></section><section id="foundry-workspace"'), 'workspace is a sibling outside the sign-in ceremony');
  for (const id of ['search','mode-schema','mode-objects','graph','model-table','zoom-in','zoom-out','fit','link-properties']) assert.ok(input.html.includes(`id="live-ontology-${id}"`));
  assert.ok(input.html.includes('href="#foundry-workspace"'));
  assert.ok(input.html.includes('href="#ontology-command-console"'), 'sample graph remains explicitly reachable');
  assert.ok(input.html.includes('This view is not an atomic model snapshot.'));
  const workspaceCss=input.css.slice(input.css.indexOf('.foundry-workspace {'));
  const desktopColumns=workspaceCss.match(/\.foundry-workspace-body \{[^}]*grid-template-columns:\s*([^;]+);/)[1];
  assert.match(desktopColumns,/13rem minmax\(0, 1fr\) minmax\(19rem, \.7fr\)/);
  const collapseAt=Number(workspaceCss.match(/@media \(max-width: ([\d.]+)rem\)/)[1])*16;
  assert.ok(1280>collapseAt, '1280 desktop retains simultaneous library, graph and inspector');
  assert.ok(1024<=collapseAt, 'tablet can collapse the inspector below the graph');
  assert.ok(workspaceCss.includes('@media (max-width: 60rem)'), '390 mobile receives a single-column layout');
  const disclosureTags=[...input.html.matchAll(/<details\b[^>]*>/g)].map(match=>match[0]);
  for(const className of ['foundry-library','foundry-inspector'])assert.ok(disclosureTags.some(tag=>tag.includes(`class="${className}"`) && /\sopen(?:[\s=>])/.test(tag)), 'panes remain open operable native disclosures independent of attribute order');
  assert.ok(workspaceCss.includes('overflow-x: auto')&&workspaceCss.includes('min-height: max(44px, 2.75rem)'), 'table overflow and unscaled touch controls are preserved');
  assert.ok(!input.html.includes('id="live-ontology-publish"'), 'unqualified authoring is not exposed');
  await flush(); find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  const types = () => find('live-ontology-library-types').querySelectorAll('[data-model-type]');
  const relations = () => find('live-ontology-library-links').querySelectorAll('[data-model-link]');
  const cards = () => find('live-ontology-scene').querySelectorAll('[data-model-type]');
  const edges = () => find('live-ontology-scene').querySelectorAll('[data-model-edge]');
  const link = {link_type:'lty_reports',from_entity_type:'ety_profile',to_entity_type:'ety_profile',cardinality:'one_to_many',allow_cross_tenant:false,revision:0};
  assert.equal(types().length,2); assert.equal(cards().length,2); assert.equal(edges().length,0);
  assert.equal(cards()[0].children.find(child=>child.tagName==='strong').textContent,'Record','native display name is the primary node heading');
  assert.match(text(types()[0]),/Record.*ety_record/,'stable type ID remains secondary');
  assert.match(find('live-ontology-type').children[0].textContent,/Record · ety_record/);
  assert.match(find('live-ontology-schema-summary').textContent,/Record · ety_record/);
  assert.match(text(cards()[1]),/직원 <img src=x onerror=alert\(1\)>/,'native labels are literal text');
  assert.ok(!cards()[1].children.some(child=>child.tagName==='img'),'label content never creates HTML');
  find('live-ontology-search').value='직원';find('live-ontology-search').emit('input');assert.equal(types().length,1,'native display names are searchable');
  find('live-ontology-search').value='';find('live-ontology-search').emit('input');
  assert.match(text(find('live-ontology-library-links')),/No relationship declarations/);
  assert.match(find('live-ontology-model-summary').textContent,/2 type declarations · 0 relationship declarations/);
  nativeLinks=[link,{...link,link_type:'lty_has',from_entity_type:'ety_record',cardinality:'many_to_many',revision:3,allow_cross_tenant:true}];
  find('live-ontology-types-refresh').emit('click');await flush();
  assert.equal(edges().length,2);assert.equal(relations().length,2);
  const self=edges().find(edge=>edge.dataset.modelEdge==='lty_reports'); assert.match(self.attrs.d,/ C /,'self-relationship has a curved native edge');
  const profile=cards().find(card=>card.dataset.modelType==='ety_profile');profile.focus();profile.emit('click');
  assert.equal(find('live-ontology-type').value,'ety_profile');assert.equal(document.activeElement,profile,'selection keeps its invoking canvas control');
  assert.equal(profile.attrs['aria-pressed'],'true');
  assert.match(text(find('live-ontology-schema-properties')),/PII_IDENTIFYING/);
  assert.match(find('live-ontology-schema-summary').textContent,/Primary-key property: id.*Title property: <script>label/);
  relations()[0].focus();relations()[0].emit('click');
  assert.equal(find('live-ontology-schema').hidden,true);assert.equal(find('live-ontology-link').hidden,false);
  assert.match(text(find('live-ontology-link-properties')),/lty_reports.*ety_profile.*one → many \(one_to_many\).*false.*revision 0/);
  cards()[0].emit('click');assert.equal(find('live-ontology-link').hidden,true);assert.equal(find('live-ontology-link-properties').children.length,0);
  find('live-ontology-search').value='flags';find('live-ontology-search').emit('input');
  assert.equal(types().length,1,'search includes recursive declared properties');
  find('live-ontology-search').value='<script>label';find('live-ontology-search').emit('input');assert.equal(types().length,1);
  assert.equal(cards().length,2,'library filtering does not silently delete model nodes');
  find('live-ontology-search').value='reports';find('live-ontology-search').emit('input');assert.equal(relations().length,1);assert.equal(types().length,0);
  relations()[0].focus();find('live-ontology-search').value='absent';find('live-ontology-search').emit('input');assert.equal(document.activeElement,find('live-ontology-search'),'filtered-away library focus returns to search');
  find('live-ontology-search').value='';find('live-ontology-search').emit('input');
  find('live-ontology-view-table').emit('click');assert.equal(find('live-ontology-graph').hidden,true);assert.equal(find('live-ontology-model-table').hidden,false);
  assert.match(text(find('live-ontology-model-table')),/Relationship declarations.*lty_reports.*one → many/);
  const tableType=find('live-ontology-model-table').querySelectorAll('[data-model-type]')[0];tableType.focus();find('live-ontology-view-graph').emit('click');assert.equal(document.activeElement,find('live-ontology-view-graph'),'hidden table releases focus to the representation control');
  find('live-ontology-graph').focus();find('live-ontology-graph').emit('keydown',{key:'ArrowRight'});assert.equal(find('live-ontology-graph').scrollLeft,80);
  find('live-ontology-graph').emit('keydown',{key:'+'});assert.equal(find('live-ontology-zoom').textContent,'125%');
  find('live-ontology-graph').emit('keydown',{key:'Home'});assert.equal(find('live-ontology-graph').scrollLeft,0);assert.notEqual(find('live-ontology-zoom').textContent,'125%');
  find('live-ontology-zoom-in').focus();for(let i=0;i<10;i++)find('live-ontology-zoom-in').emit('click');
  assert.equal(find('live-ontology-zoom').textContent,'200%');assert.equal(find('live-ontology-zoom-in').disabled,true);assert.equal(document.activeElement,find('live-ontology-fit'),'zoom bound restores focus before leaving a disabled control');
  find('live-ontology-mode-objects').emit('click');assert.equal(find('live-ontology-model').hidden,true);assert.equal(find('live-ontology-objects').hidden,false);
  find('live-ontology-query').focus();find('live-ontology-mode-schema').emit('click');assert.equal(document.activeElement,find('live-ontology-mode-schema'),'hidden Objects form releases focus');
  const decodeLinks=rows=>vm.runInNewContext(`ontologyDecode.links(${JSON.stringify(rows)},${JSON.stringify(schemaFixture)})`,context);
  const decodeLabels=rows=>vm.runInNewContext(`ontologyDecode.labels(${JSON.stringify(rows)},${JSON.stringify(schemaFixture)})`,context);
  assert.doesNotThrow(()=>decodeLabels(labelFixture));
  assert.doesNotThrow(()=>decodeLabels(labelFixture.map(row=>({...row,display_name:'\uFEFF'}))),'native Unicode non-whitespace text remains legal');
  for(const malformed of [null,[],[labelFixture[0],labelFixture[0]],[[labelFixture[0].entity_type,1,'Record']],...[
    row=>delete row.entity_type,row=>delete row.revision,row=>delete row.display_name,row=>row.extra=true,
    row=>row.entity_type='ety_missing',row=>row.revision=2,row=>row.revision=-1,row=>row.revision=4294967296,row=>row.display_name='\u2003',
  ].map(change=>{const rows=JSON.parse(JSON.stringify(labelFixture));change(rows[0]);return rows;})])assert.throws(()=>decodeLabels(malformed),/source_unavailable/,'missing, malformed, orphan or inconsistent labels fail closed');
  for(const cardinality of ['one_to_one','one_to_many','many_to_many'])assert.doesNotThrow(()=>decodeLinks([{...link,cardinality}]));
  assert.doesNotThrow(()=>decodeLinks([{...link,link_type:'lty_Ω<script>'}]));assert.doesNotThrow(()=>decodeLinks([]));
  for(const malformed of [null,[[link.link_type,link.from_entity_type,link.to_entity_type,link.cardinality,false,0]],[link,link],...[
    row=>delete row.revision,row=>delete row.allow_cross_tenant,row=>row.extra=true,row=>row.cardinality='many_to_one',row=>row.revision=-1,row=>row.revision=4294967296,row=>row.allow_cross_tenant='true',row=>row.link_type='lty_',row=>row.to_entity_type='ety_missing',
  ].map(change=>{const row={...link};change(row);return [row];})])assert.throws(()=>decodeLinks(malformed),/source_unavailable/);
  nativeLinks=[link];find('live-ontology-types-refresh').emit('click');await flush();
  cards()[0].focus();nativeLinks=[{...link,to_entity_type:'ety_missing'}];find('live-ontology-types-refresh').emit('click');await flush();
  assert.match(find('live-ontology-status').textContent,/unavailable/);assert.equal(cards().length,0);assert.equal(types().length,0);assert.equal(edges().length,0);assert.equal(find('live-ontology-link-properties').children.length,0);
  assert.equal(document.activeElement,find('live-ontology-types-refresh'),'missing endpoint refusal restores enabled focus');
  nativeLinks=[link];await restore();holdPath='/link-types';find('live-ontology-types-refresh').emit('click');await flush();
  assert.equal(cards().length,0,'no model is shown before both separate catalog reads complete');
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');held();await flush();
  assert.equal(root.hidden,true);assert.equal(cards().length,0);assert.equal(types().length,0);assert.equal(relations().length,0);assert.equal(find('live-ontology-link-properties').children.length,0,'late previous-account links cannot restore any model surface');
  find('sign-in-account').value='usr_alice';await restore();holdPath='/link-types';find('live-ontology-types-refresh').emit('click');await flush();find('sign-in-account').focus();held();await flush();
  assert.equal(document.activeElement,find('sign-in-account'),'late model does not steal newer outside focus');
  nativeError='forbidden';find('live-ontology-types-refresh').emit('click');await flush();assert.equal(cards().length,0);assert.equal(find('live-ontology-library-summary').textContent,'');
  nativeError=null;await restore();relations()[0].emit('click');find('session-logout').emit('click');
  assert.equal(root.hidden,true);assert.equal(cards().length,0);assert.equal(relations().length,0);assert.equal(find('live-ontology-link-properties').children.length,0);assert.equal(find('live-ontology-search').value,'');
  assert.equal(find('live-ontology-access-notice').hidden,false);assert.equal(find('live-ontology-mode-schema').attrs['aria-pressed'],'true');
  await flush();await restore();nativeLabels=[{...labelFixture[0],display_name:' '},labelFixture[1]];find('live-ontology-types-refresh').emit('click');await flush();
  assert.match(find('live-ontology-status').textContent,/unavailable/);assert.equal(cards().length,0);assert.equal(types().length,0);assert.equal(find('live-ontology-schema-summary').textContent,'');
  nativeLabels=null;await restore();holdPath='/type-labels';find('live-ontology-types-refresh').emit('click');await flush();
  assert.equal(cards().length,0,'refresh hides native names before all catalog reads complete');
  assert.ok(!text(find('live-ontology-type')).includes('Record')&&!text(find('live-ontology-type')).includes('직원'),'pending chooser preserves IDs while clearing prior names');
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');held();await flush();
  assert.equal(root.hidden,true);assert.equal(cards().length,0);assert.equal(types().length,0);assert.equal(find('live-ontology-schema-summary').textContent,'','late old-account labels cannot repopulate the inspector');
  find('sign-in-account').value='usr_alice';
  await flush();nativeSchemas=[schemaFixture[0]];nativeLinks=[];await restore();
  assert.equal(cards().length,1);assert.equal(edges().length,0);assert.match(text(find('live-ontology-scene')),/ety_record.*Legacy string contract/,'served Record-only model needs no fictional nodes');
  nativeSchemas=[{...schemaFixture[0],revision:0}];find('live-ontology-types-refresh').emit('click');await flush();assert.match(find('live-ontology-schema-summary').textContent,/declaration revision 0/);
  assert.ok(calls.filter(([url])=>url.endsWith('/link-types')||url.endsWith('/type-labels')).every(([,options])=>options.credentials==='same-origin'&&options.cache==='no-store'&&options.redirect==='error'));
  console.log('Native workspace schema/link library, graph/table/viewport, inspector, focus and session refusal contracts PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"##;

#[test]
fn native_workspace_composes_registered_model_and_preserves_trust_and_accessibility() {
    super::check_controller(CHECK);
}

const CHECK_FIT: &str = r##"
(async () => {
  assert.match(input.css,/\.foundry-type-node \{[^}]*width: 260px; height: 200px;/);
  assert.match(input.css,/\.foundry-edges > path \{[^}]*stroke-width: 1\.5;/);
  assert.match(input.css,/\.foundry-edge-label \{[^}]*max-width: 160px;/);
  await flush();find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';
  const viewport=find('live-ontology-graph'),scene=find('live-ontology-scene'),stage=find('live-ontology-stage');
  const cards=()=>scene.querySelectorAll('[data-model-type]'),labels=()=>scene.querySelectorAll('[data-model-link]'),edges=()=>scene.querySelectorAll('[data-model-edge]');
  const close=(actual,expected)=>assert.ok(Math.abs(actual-expected)<1e-10,`${actual} ≠ ${expected}`);
  const zoom=()=>Number(scene.style.transform.match(/^scale\(([^)]+)\)$/)[1]);
  async function model(rows,relations=[]) {
    nativeSchemas=rows;nativeLabels=rows.map(row=>({entity_type:row.entity_type,revision:row.revision,display_name:row.entity_type}));nativeLinks=relations;await restore();
    for(const card of cards()){card.mockWidth=260;card.mockHeight=200;}
    for(const label of labels()){label.mockWidth=160;label.mockHeight=60;}
    for(const edge of edges())edge.mockStrokeWidth=1.5;
  }
  function fit(width,height,expected) {
    viewport.clientWidth=width;viewport.clientHeight=height;viewport.scrollLeft=80;viewport.scrollTop=80;
    find('live-ontology-fit').emit('click');close(zoom(),expected);
    assert.equal(viewport.scrollLeft,0);assert.equal(viewport.scrollTop,0);
    close(Number.parseFloat(stage.style.width),Number.parseFloat(scene.style.width)*expected);
    close(Number.parseFloat(stage.style.height),Number.parseFloat(scene.style.height)*expected);
  }
  const record=schemaFixture[0];await model([record]);
  assert.equal(scene.style.width,'640px');assert.equal(scene.style.height,'360px');
  assert.equal(cards()[0].style.left,'70px');assert.equal(cards()[0].style.top,'60px');
  cards()[0].focus();cards()[0].emit('click');const selected=cards()[0];
  fit(390,400,1);assert.equal(document.activeElement,selected);assert.equal(selected.attrs['aria-pressed'],'true');
  fit(360,400,328/330);fit(200,400,168/330);fit(390,200,168/260);fit(80,400,.25);
  assert.equal(find('live-ontology-zoom-out').disabled,true);
  for(let i=0;i<10;i++)find('live-ontology-zoom-in').emit('click');close(zoom(),2);
  assert.equal(stage.style.width,'1280px');assert.equal(stage.style.height,'720px');
  assert.equal(find('live-ontology-zoom-in').disabled,true);
  viewport.focus();viewport.emit('keydown',{key:'ArrowRight'});viewport.emit('keydown',{key:'ArrowDown'});
  assert.equal(viewport.scrollLeft,80);assert.equal(viewport.scrollTop,80);
  viewport.clientWidth=390;viewport.clientHeight=400;viewport.emit('keydown',{key:'Home'});close(zoom(),1);
  assert.equal(viewport.scrollLeft,0);assert.equal(viewport.scrollTop,0);assert.equal(document.activeElement,viewport);
  assert.equal(selected.attrs['aria-pressed'],'true');assert.equal(selected.style.transform,undefined);
  const rows=Array.from({length:4},(_,index)=>({...record,entity_type:`ety_card${index}`}));await model(rows);
  assert.equal(scene.style.width,'800px');assert.equal(scene.style.height,'620px');
  assert.deepEqual(cards().map(card=>[card.style.left,card.style.top]),[['70px','60px'],['420px','60px'],['70px','320px'],['420px','320px']]);
  fit(720,600,1);fit(390,400,358/680);
  cards()[3].mockHeight=300;fit(1000,600,568/620);
  const self={link_type:'lty_self',from_entity_type:record.entity_type,to_entity_type:record.entity_type,cardinality:'one_to_many',allow_cross_tenant:false,revision:1};
  await model([record],[self]);
  // Explicit platform bounding-box input for the generated cubic; no CSS layout is simulated.
  edges()[0].mockBounds={x:330,y:89.72169679450217,width:56.25,height:88.11216196655123};
  assert.equal(edges()[0].attrs.d,'M 330 100 C 405 50, 405 200, 330 175');
  assert.equal(labels()[0].style.left,'335px');assert.equal(labels()[0].style.top,'200px');
  for(let i=0;i<10;i++)find('live-ontology-zoom-in').emit('click');close(zoom(),2);
  const labelFit=(358-160)/335;fit(390,400,labelFit);
  close(335*zoom()+labels()[0].offsetWidth,358);assert.equal(labels()[0].style.transform,`scale(${1/labelFit})`);
  for(let i=0;i<10;i++)find('live-ontology-zoom-out').emit('click');close(zoom(),.25);
  fit(390,400,labelFit);fit(530,400,1);
  labels()[0].mockHeight=220;fit(1000,400,(368-220)/200);
  close(200*zoom()+labels()[0].offsetHeight,368);
  const reverse={...self,link_type:'lty_reverse',from_entity_type:'ety_card2',to_entity_type:'ety_card0'};await model(rows,[reverse]);
  edges()[0].mockBounds={x:200,y:57.88082727868517,width:0,height:464.2383454426297};
  assert.equal(edges()[0].attrs.d,'M 200 520 C 200 560, 200 20, 200 60');
  const pathBottom=522.1191727213148+Math.hypot(7,4)*1.5;fit(1024,560,528/pathBottom);
  assert.ok(zoom()<1,'stroke/marker room below a reverse path affects Fit even when every node and label fits');
  await model([]);fit(390,400,358/640);
  assert.equal(cards().length,0);assert.equal(edges().length,0);assert.equal(labels().length,0);
  nativeSchemas=[record];nativeLabels=[{entity_type:record.entity_type,revision:record.revision,display_name:'Record'}];nativeLinks=[];await restore();
  cards()[0].mockWidth=260;cards()[0].mockHeight=200;fit(390,400,1);
  find('session-logout').emit('click');await flush();fit(390,400,358/640);
  assert.equal(cards().length,0);assert.equal(edges().length,0);assert.equal(labels().length,0,'clearing discards all occupied-content measurements');
  console.log('Content Fit, target-zoom labels, path bounds and unchanged scene/zoom/pan/selection contracts PASS; explicit mock geometry only');
})().catch(error=>{console.error(error);process.exitCode=1;});
"##;

#[test]
fn canvas_fit_uses_occupied_content_and_preserves_manual_viewport_controls() {
    super::check_controller(CHECK_FIT);
}

const CHECK_TARGET_HEIGHT: &str = r##"
const targetSelectors=[
  '.foundry-workspace button, .foundry-workspace summary',
  '.foundry-mode-bar select',
  '.foundry-library input',
  '.live-ontology-query input, .live-ontology-query select',
];
function minimumAt(expression,rootPixels) {
  const terms=expression.startsWith('max(')?expression.slice(4,-1).split(','):[expression];
  return Math.max(...terms.map(term=>{
    const value=term.trim().match(/^(\d+(?:\.\d+)?)(px|rem)$/);assert.ok(value,'bounded length expression');
    return Number(value[1])*(value[2]==='rem'?rootPixels:1);
  }));
}
for(const selector of targetSelectors) {
  const start=input.css.indexOf(`${selector} {`);assert.ok(start>=0,`${selector} exists`);
  const block=input.css.slice(start,input.css.indexOf('}',start));
  const expression=block.match(/min-height:\s*([^;]+);/)[1].trim();
  for(const [rootPixels,expectedPixels] of [[13,44],[16,44],[20,55],[32,88]]) {
    assert.equal(minimumAt(expression,rootPixels),expectedPixels,`${selector} at root ${rootPixels}px retains the physical floor and font scaling`);
  }
}
assert.match(input.css,/html,\s*body \{\s*font-size: var\(--text-sm\);/,'global typography remains unchanged');
console.log('Workspace control height floor at13/16px and font growth at20/32px PASS; source expression proof only');
"##;

#[test]
fn workspace_controls_keep_physical_height_and_grow_with_root_font() {
    super::check_controller(CHECK_TARGET_HEIGHT);
}
