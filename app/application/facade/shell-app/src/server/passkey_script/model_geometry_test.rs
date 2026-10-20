use super::explorer_test::check_controller;

#[test]
fn fixed_incident_routes_preserve_every_link_without_shared_interior_segments() {
    check_controller(
        r#"
(async()=>{
  await flush();
  const types=Array.from({length:8},(_,i)=>({...schemaFixture[0],entity_type:`ety_${i}`,display_name:'Type '+i}));
  const rows=[['direct_0',0,1],['direct_1',0,2],['direct_2',0,3],['direct_3',4,0],['direct_4',0,5],['parallel',0,1],['reverse',1,0],['self',0,0],['induced',2,3],['unrelated',6,7],['unrelated_reverse',7,6]];
  const links=rows.map(([link_type,from,to])=>({link_type,from_entity_type:`ety_${from}`,to_entity_type:`ety_${to}`,cardinality:'one_to_many',revision:1,allow_cross_tenant:false}));
  const sizes={types:new Map(types.map(row=>[row.entity_type,{w:182,h:68}])),links:new Map(links.map(row=>[row.link_type,{w:96,h:44}]))};
  const plan=context.ontologyGraphPlan(types,links,{type:'ety_0',link:null},sizes,{width:1800,height:1600,padding:13},'all');
  assert.ok(plan);assert.deepEqual(Array.from(plan.nodes,row=>row.id).sort(),types.map(row=>row.entity_type).sort());
  assert.deepEqual(Array.from(plan.edges,row=>row.id).sort(),links.map(row=>row.link_type).sort(),'all eleven native links have exactly one route');
  const epsilon=1e-6,segments=edge=>edge.points.slice(1).map((b,i)=>[edge.points[i],b]).filter(([a,b])=>Math.hypot(b[0]-a[0],b[1]-a[1])>epsilon);
  const cards=new Map(plan.nodes.map(row=>[row.id,row])),overlap=(a,b)=>a.x<b.x+b.w-epsilon && b.x<a.x+a.w-epsilon && a.y<b.y+b.h-epsilon && b.y<a.y+a.h-epsilon;
  for(const edge of plan.edges){
    const native=links.find(row=>row.link_type===edge.id),from=cards.get(native.from_entity_type),to=cards.get(native.to_entity_type),first=edge.points[0],last=edge.points.at(-1);
    assert.equal(edge.from,native.from_entity_type);assert.equal(edge.to,native.to_entity_type);
    const onCard=(point,card)=>point && point[0]>=card.x-epsilon && point[0]<=card.x+card.w+epsilon && point[1]>=card.y-epsilon && point[1]<=card.y+card.h+epsilon;
    assert.ok(onCard(first,from) && onCard(last,to),'source and directed arrow retain native endpoints');
    const y=edge.label.y+edge.label.h/2;
    assert.ok(segments(edge).some(([a,b])=>Math.abs(a[1]-y)<epsilon && Math.abs(b[1]-y)<epsilon && Math.min(a[0],b[0])<=edge.label.x+epsilon && Math.max(a[0],b[0])>=edge.label.x+edge.label.w-epsilon),'label stays on its own route');
    for(const point of edge.points)assert.ok(point.every(Number.isFinite) && point[0]>=0 && point[0]<=plan.width && point[1]>=0 && point[1]<=plan.height,'finite route stays in scene bounds');
    for(const card of plan.nodes){
      assert.ok(!overlap(edge.label,card),'label avoids cards');
      for(const[a,b]of segments(edge)){
        assert.ok(Math.abs(a[0]-b[0])<epsilon || Math.abs(a[1]-b[1])<epsilon,'route is orthogonal');
        const crosses=Math.abs(a[0]-b[0])<epsilon?a[0]>card.x+epsilon && a[0]<card.x+card.w-epsilon && Math.max(a[1],b[1])>card.y+epsilon && Math.min(a[1],b[1])<card.y+card.h-epsilon:a[1]>card.y+epsilon && a[1]<card.y+card.h-epsilon && Math.max(a[0],b[0])>card.x+epsilon && Math.min(a[0],b[0])<card.x+card.w-epsilon;
        assert.ok(!crosses,'route avoids card interiors');
      }
    }
  }
  let pairs=0;
  for(let i=0;i<plan.edges.length;i++)for(let j=i+1;j<plan.edges.length;j++){
    pairs++;
    assert.ok(!overlap(plan.edges[i].label,plan.edges[j].label),'route labels retain separate lanes');
    for(const[a,b]of segments(plan.edges[i]))for(const[c,d]of segments(plan.edges[j])){
      const horizontal=Math.abs(a[1]-b[1])<epsilon && Math.abs(c[1]-d[1])<epsilon && Math.abs(a[1]-c[1])<epsilon;
      const vertical=Math.abs(a[0]-b[0])<epsilon && Math.abs(c[0]-d[0])<epsilon && Math.abs(a[0]-c[0])<epsilon;
      const axis=horizontal?0:vertical?1:null;
      if(axis!==null)assert.ok(Math.min(Math.max(a[axis],b[axis]),Math.max(c[axis],d[axis]))-Math.max(Math.min(a[axis],b[axis]),Math.min(c[axis],d[axis]))<=epsilon,`${plan.edges[i].id}/${plan.edges[j].id} share an interior segment: ${JSON.stringify([a,b,c,d])}`);
    }
  }
  assert.equal(pairs,55,'every fixed input-derived pair is checked without an availability skip');
  console.log('Fixed8nodes/11links/55unconditional route pairs: singleton,parallel,reverse,self,induced,unrelated PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn measured_scope_routes_keep_identity_direction_label_association_and_capacity() {
    check_controller(
        r#"
(async()=>{
  await flush();const plan=context.ontologyGraphPlan;assert.equal(typeof plan,'function');
  const types=Array.from({length:52},(_,i)=>({...schemaFixture[0],entity_type:`ety_${i}`,display_name:i?'Related '+i:'직원 Employee 🧭'}));
  const link=(id,from,to)=>({link_type:id,from_entity_type:from,to_entity_type:to,cardinality:'one_to_many',revision:1,allow_cross_tenant:false});
  const links=[...Array.from({length:7},(_,i)=>link('direct_'+i,i===4?`ety_${i+1}`:'ety_0',i===4?'ety_0':`ety_${i+1}`)),link('self','ety_0','ety_0'),link('induced','ety_2','ety_3'),...Array.from({length:139},(_,i)=>link('other_'+i,`ety_${8+i%44}`,`ety_${8+(i*7+1)%44}`))];
  const sizes={types:new Map(types.map(t=>[t.entity_type,{w:182,h:68}])),links:new Map(links.map(l=>[l.link_type,{w:96,h:44}]))};
  const projected=context.ontologyModelProjection(types,links,{type:'ety_0',link:null},'related');
  const overlap=(a,b)=>a.x<b.x+b.w-.5 && b.x<a.x+a.w-.5 && a.y<b.y+b.h-.5 && b.y<a.y+a.h-.5;
  function verify(p,eligible){
    assert.ok(p);const cards=new Map(p.nodes.map(n=>[n.id,n]));assert.equal(new Set(p.edges.map(e=>e.id)).size,p.edges.length);assert.equal(new Set(p.edges.map(e=>e.path)).size,p.edges.length,'parallel and self paths are distinct');
    for(const edge of p.edges){
      const native=eligible.find(l=>l.link_type===edge.id);assert.equal(edge.from,native.from_entity_type);assert.equal(edge.to,native.to_entity_type);
      const from=cards.get(edge.from),to=cards.get(edge.to),first=edge.points[0],last=edge.points.at(-1);
      assert.ok(first[0]>=from.x-.01 && first[0]<=from.x+from.w+.01 && first[1]>=from.y-.01 && first[1]<=from.y+from.h+.01,'path starts on its native source');
      assert.ok(last[0]>=to.x-.01 && last[0]<=to.x+to.w+.01 && last[1]>=to.y-.01 && last[1]<=to.y+to.h+.01,'arrow ends on its native target');
      const y=edge.label.y+edge.label.h/2;assert.ok(edge.points.some((point,index)=>index && Math.abs(point[1]-y)<.01 && Math.abs(edge.points[index-1][1]-y)<.01 && Math.min(point[0],edge.points[index-1][0])<=edge.label.x+.01 && Math.max(point[0],edge.points[index-1][0])>=edge.label.x+edge.label.w-.01),'each label occupies its own actual horizontal path segment');
      for(const card of p.nodes)assert.ok(!overlap(edge.label,card),'relationship label never covers a type card');
      for(let segment=1;segment<edge.points.length;segment++){const a=edge.points[segment-1],b=edge.points[segment];for(const card of p.nodes){const crosses=a[0]===b[0]?a[0]>card.x+.01 && a[0]<card.x+card.w-.01 && Math.max(a[1],b[1])>card.y+.01 && Math.min(a[1],b[1])<card.y+card.h-.01:a[1]>card.y+.01 && a[1]<card.y+card.h-.01 && Math.max(a[0],b[0])>card.x+.01 && Math.min(a[0],b[0])<card.x+card.w-.01;assert.ok(!crosses,'routes never cross a card interior');}}
      for(const point of edge.points)assert.ok(point[0]>=0 && point[1]>=0 && point[0]<=p.width && point[1]<=p.height,'all route points remain inside their scene');
    }
    for(let i=0;i<p.edges.length;i++)for(let j=i+1;j<p.edges.length;j++)assert.ok(!overlap(p.edges[i].label,p.edges[j].label),'labels have separate readable lanes');
  }
  function verifyEveryPair(p){
    let pairs=0;const epsilon=1e-6;
    for(let i=0;i<p.edges.length;i++)for(let j=i+1;j<p.edges.length;j++){
      pairs++;
      for(let k=1;k<6;k++)for(let l=1;l<6;l++){
        const [a,b,c,d]=[p.edges[i].points[k-1],p.edges[i].points[k],p.edges[j].points[l-1],p.edges[j].points[l]];
        const horizontal=Math.abs(a[1]-b[1])<epsilon && Math.abs(c[1]-d[1])<epsilon && Math.abs(a[1]-c[1])<epsilon;
        const vertical=Math.abs(a[0]-b[0])<epsilon && Math.abs(c[0]-d[0])<epsilon && Math.abs(a[0]-c[0])<epsilon,axis=horizontal?0:vertical?1:null;
        if(axis!==null)assert.ok(Math.min(Math.max(a[axis],b[axis]),Math.max(c[axis],d[axis]))-Math.max(Math.min(a[axis],b[axis]),Math.min(c[axis],d[axis]))<=epsilon,'native routes share no positive-length collinear segment');
      }
    }
    assert.equal(pairs,55,'all native eleven-link pairs are checked unconditionally');
  }
  function markerIntersections(p){
    const cross=(a,b,c)=>(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0]);
    const on=(a,b,c)=>Math.abs(cross(a,b,c))<1e-6 && c.every((value,i)=>value>=Math.min(a[i],b[i])-1e-6 && value<=Math.max(a[i],b[i])+1e-6);
    const intersects=(a,b,c,d)=>cross(a,b,c)*cross(a,b,d)<0 && cross(c,d,a)*cross(c,d,b)<0 || on(a,b,c) || on(a,b,d) || on(c,d,a) || on(c,d,b);
    const distance=(point,a,b)=>{const dx=b[0]-a[0],dy=b[1]-a[1],t=Math.max(0,Math.min(1,((point[0]-a[0])*dx+(point[1]-a[1])*dy)/(dx*dx+dy*dy || 1)));return Math.hypot(point[0]-a[0]-t*dx,point[1]-a[1]-t*dy);};
    const inside=(point,triangle)=>{const turns=triangle.map((a,i)=>cross(a,triangle[(i+1)%3],point));return turns.every(n=>n>=-1e-6) || turns.every(n=>n<=1e-6);};
    const found=[];
    for(const arrow of p.edges){
      const [x,y]=arrow.points.at(-1),direction=Math.sign(x-arrow.points.at(-2)[0]);
      const triangle=[[x-10.5*direction,y-6],[x+1.5*direction,y],[x-10.5*direction,y+6]];
      for(const route of p.edges){
        if(route.id===arrow.id)continue;
        const hit=route.points.slice(1).some((b,i)=>{const a=route.points[i];return inside(a,triangle) || inside(b,triangle) || triangle.some((c,j)=>{const d=triangle[(j+1)%3];return intersects(a,b,c,d) || Math.min(distance(a,c,d),distance(b,c,d),distance(c,a,b),distance(d,a,b))<=.75+1e-6;});});
        if(hit)found.push([arrow.id,route.id]);
      }
    }
    return found.sort();
  }
  let p=plan(projected.schemas,projected.links,{type:'ety_0',link:null},sizes,{width:849.5,height:534.53125,padding:13},'related');verify(p,links);assert.equal(p.nodes.length,8);assert.equal(p.edges.length,9);assert.ok(p.width<=849.5 && p.height<=534.53125,'eight-node composition is a measured result, not a fixed limit');
  for(const width of [849.5,506,720,342]){p=plan(projected.schemas,projected.links,{type:'ety_0',link:null},sizes,{width,height:323,padding:13},'related');verify(p,links);assert.ok(p.height<=323 || p.neighbors===0);}
  // Native Employee neighborhood topology; dimensions are synthetic canonical fixture sizes, not saved DOM measurements.
  const nativeIds=['employee','department','employment_contract','leave_balance','leave_policy','leave_request','position','team'].map(id=>'ety_'+id);
  const nativeTypes=nativeIds.map(entity_type=>({...schemaFixture[0],entity_type}));
  const nativeRows=[['member_of',0,1],['employee_contract',0,2],['employee_balance',0,3],['employee_policy',0,4],['employee_request',0,5],['employee_position',0,6],['employee_team',0,7],['employee_manages',0,0],['employee_assignment',0,1],['department_employs',1,0],['contract_governs',2,4]];
  const nativeLinks=nativeRows.map(([id,from,to])=>link('lty_'+id,nativeIds[from],nativeIds[to]));
  nativeLinks.forEach((row,i)=>{row.cardinality=[0,3,5,6,8,10].includes(i)?'many_to_many':'one_to_many';});
  const nativeSizes={types:new Map(nativeIds.map(id=>[id,{w:182,h:68}])),links:new Map(nativeLinks.map(row=>[row.link_type,{w:96,h:44}]))};
  const nativeScope=context.ontologyModelProjection(nativeTypes,nativeLinks,{type:'ety_employee',link:null},'related');
  assert.deepEqual(Array.from(nativeScope.schemas,row=>row.entity_type),nativeIds);assert.deepEqual(Array.from(nativeScope.links,row=>row.link_type),nativeLinks.map(row=>row.link_type));
  for(const height of [534.53125,511])for(const mode of ['related','all']){
    const nativePlan=plan(nativeScope.schemas,nativeScope.links,{type:'ety_employee',link:null},nativeSizes,{width:849.5,height,padding:13},mode);verify(nativePlan,nativeLinks);verifyEveryPair(nativePlan);
    assert.deepEqual(Array.from(nativePlan.nodes,row=>row.id).sort(),[...nativeIds].sort(),'retain all eight native types including Team');
    assert.deepEqual(Array.from(nativePlan.edges,row=>row.id),nativeLinks.map(row=>row.link_type),'retain all eleven native relationships');
    assert.equal(nativePlan.columns,3);assert.ok(nativePlan.width<=849.5 && nativePlan.height<=height,'native topology fits both synthetic viewport heights');
  }
  // Integer offset measurements retained from the native neutral episode, not raster-rounded card rectangles.
  const savedSizes={types:new Map(nativeIds.map(id=>[id,{w:182,h:68}])),links:new Map(nativeLinks.map((row,i)=>[row.link_type,{w:[94,88,91,94,85,94,94,85,94,85,94][i],h:46}]))};
  const margin=13+Math.hypot(7,4)*1.5,near=(actual,expected)=>assert.ok(Math.abs(actual-expected)<1e-6,`${actual} != ${expected}`);
  for(const mode of ['related','all']){
    const before=JSON.stringify([nativeTypes,nativeLinks]);
    const saved=plan(nativeTypes,nativeLinks,{type:'ety_employee',link:null},savedSizes,{width:848,height:509,padding:13},mode);verify(saved,nativeLinks);verifyEveryPair(saved);
    assert.equal(JSON.stringify([nativeTypes,nativeLinks]),before,'native declarations, cardinalities and direction remain unchanged');
    assert.deepEqual(Array.from(saved.nodes,row=>row.id).sort(),[...nativeIds].sort());assert.deepEqual(Array.from(saved.edges,row=>row.id),nativeLinks.map(row=>row.link_type));
    near(saved.width,840.1867732448957);near(saved.height,503.1867732448957);assert.ok(saved.width<=848 && saved.height<=509);assert.equal(saved.columns,3);
    for(const node of saved.nodes){const i=nativeIds.indexOf(node.id);near(node.x-margin,[306,0,608,0,608,0,608,0][i]);near(node.y-margin,[192.5,41.5,82,164,163,245,244,326][i]);assert.equal(node.w,182);assert.equal(node.h,68);}
    const intersections=markerIntersections(saved),employeeArrows=new Set(nativeLinks.filter(row=>row.to_entity_type==='ety_employee').map(row=>row.link_type));
    assert.deepEqual(intersections.filter(([arrow])=>employeeArrows.has(arrow)),[],'the three measured Employee arrow/foreign-stroke intersections must clear');
    assert.deepEqual(intersections,[['lty_employee_policy','lty_contract_governs'],['lty_employee_position','lty_contract_governs']],'the two pre-existing outer Governs intersections remain explicit');
    assert.deepEqual([saved.edges.find(row=>row.id==='lty_employee_contract').points.at(-1)[1],saved.edges.find(row=>row.id==='lty_contract_governs').points[0][1]],[131.92671995578115,150.2600532891145],'already-clear Employment Contract slots retain exact legacy coordinates');
  }
  nativeSizes.types=new Map(nativeIds.map((id,index)=>[id,{w:182-index*7,h:68}]));
  const varied=plan(nativeTypes,nativeLinks,{type:'ety_employee',link:null},nativeSizes,{width:849.5,height:511,padding:13},'all');verify(varied,nativeLinks);
  const laneGroups=new Map(),variedCards=new Map(varied.nodes.map(card=>[card.id,card]));
  for(const edge of varied.edges)for(const [id,point] of [[edge.from,edge.points[1]],[edge.to,edge.points.at(-2)]]){
    const card=variedCards.get(id),key=JSON.stringify([card.x,point[0]<card.x?'left':'right']);if(!laneGroups.has(key))laneGroups.set(key,[]);laneGroups.get(key).push(point[0]);
  }
  for(const lanes of laneGroups.values()){lanes.sort((a,b)=>a-b);for(let index=1;index<lanes.length;index++)assert.ok(lanes[index]-lanes[index-1]>=2-1e-6,'unequal card widths retain two scene pixels between competing column-side lanes');}
  p=plan(types,links,{type:'ety_0',link:null},sizes,{width:849.5,height:534.53125,padding:13},'all');verify(p,links);assert.equal(p.nodes.length,52);assert.equal(p.edges.length,148);
  const hub=Array.from({length:148},(_,i)=>link('hub_'+i,i%2?'ety_0':`ety_${i%51+1}`,i%2?`ety_${i%51+1}`:'ety_0'));sizes.links=new Map(hub.map(l=>[l.link_type,{w:104,h:44}]));
  p=plan(types,hub,{type:'ety_0',link:null},sizes,{width:342,height:323,padding:13},'related');verify(p,hub);assert.equal(p.totalNeighbors,51);assert.ok(p.neighbors<51);const initial=p.neighbors;
  const expanded=plan(types,hub,{type:'ety_0',link:null},sizes,{width:342,height:200,padding:13},'related',initial+1);verify(expanded,hub);assert.ok(expanded.neighbors>=initial+1,'explicit expansion survives smaller viewport and permits honest overflow');
  const forced=plan(types,hub,{type:'ety_0',link:'hub_101'},sizes,{width:342,height:200,padding:13},'related');verify(forced,hub);assert.ok(forced.edges.some(row=>row.id==='hub_101'),'selected relationship retains its forced target despite measured overflow');
  assert.equal(plan(types,hub,{type:'ety_0',link:null},sizes,{width:0,height:323,padding:13},'related'),null,'unknown geometry refuses a fabricated capacity');
  for(const bad of [0,-1,NaN,Infinity]){
    for(const field of ['width','height','padding'])assert.equal(plan(types,hub,{type:'ety_0',link:null},sizes,{width:848,height:509,padding:13,[field]:bad},'related'),null);
    for(const [kind,id] of [['types','ety_0'],['links','hub_0']])for(const field of ['w','h']){const invalid={...sizes,[kind]:new Map(sizes[kind])};invalid[kind].set(id,{...invalid[kind].get(id),[field]:bad});assert.equal(plan(types,hub,{type:'ety_0',link:null},invalid,{width:848,height:509,padding:13},'related'),null);}
  }
  console.log('Measured production scope plan: eight-node composition,52/148All,51-neighbor hub, self/parallel/induced path-label/direction/bounds PASS; explicit platform measurements only');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn bounded_port_allocation_preserves_unsupported_slots_and_native_order() {
    check_controller(
        r#"
(async()=>{
  await flush();const types=['hub','a','b','c'].map(id=>({...schemaFixture[0],entity_type:'ety_'+id}));
  function layout(rows,heights={},labelHeight=44){
    const links=rows.map(([link_type,from,to])=>({link_type,from_entity_type:'ety_'+from,to_entity_type:'ety_'+to,cardinality:'one_to_many',revision:1,allow_cross_tenant:false}));
    const sizes={types:new Map(types.map(row=>[row.entity_type,{w:182,h:heights[row.entity_type] || 68}])),links:new Map(links.map(row=>[row.link_type,{w:96,h:labelHeight}]))};
    const plan=context.ontologyGraphPlan(types,links,{type:'ety_hub',link:null},sizes,{width:1800,height:1600,padding:13},'all');
    assert.equal(plan.nodes.length,4);assert.equal(plan.edges.length,links.length);
    for(const edge of plan.edges)for(const point of edge.points)assert.ok(point.every(Number.isFinite),'unsupported inputs retain finite routes');
    return plan;
  }
  const node=plan=>plan.nodes.find(row=>row.id==='ety_hub');
  const left=plan=>plan.edges.flatMap(edge=>[['source',edge.from,edge.points[0]],['target',edge.to,edge.points.at(-1)]].filter(([role,id,point])=>id==='ety_hub' && point[0]===node(plan).x).map(([role,id,point])=>[edge.id,role,point[1]]));
  // Concrete baseline coordinates bind the fallback; these checks do not reproduce the new allocation.
  assert.deepEqual(left(layout([['out0','hub','a'],['out1','hub','a']])),[['out0','source',103.67671995578115],['out1','source',122.0100532891145]],'zero targets retain exact slots');
  assert.deepEqual(left(layout([['in0','a','hub']])),[['in0','target',99.59338662244782]],'zero sources retain exact slot');
  assert.deepEqual(left(layout([['in0','a','hub'],['out','hub','a'],['in1','a','hub']])),[['in0','target',124.34338662244782],['out','source',138.09338662244784],['in1','target',151.84338662244784]],'multiple incoming endpoints retain exact slots');
  const crowded=layout([...Array.from({length:13},(_,i)=>['out'+i,'hub','a']),['in','a','hub']]);
  assert.deepEqual(left(crowded).map(row=>row[2]),[392.0100532891145,395.67671995578115,399.34338662244784,403.0100532891145,406.67671995578115,410.34338662244784,414.0100532891145,417.67671995578115,421.34338662244784,425.0100532891145,428.67671995578115,432.34338662244784,436.0100532891145,439.67671995578115],'70.5px footprint does not invent space in a real68px card');
  assert.equal(node(crowded).h,68);
  const midpoint=layout([['in','a','hub'],['out','hub','a']],{'ety_c':48.5},55),out=midpoint.edges.find(row=>row.id==='out');
  assert.equal(out.label.y+out.label.h/2,node(midpoint).y+34,'source label is exactly at node midpoint');
  assert.deepEqual(left(midpoint),[['in','target',104.92671995578115],['out','source',123.2600532891145]],'a final fan direction mismatch retains exact legacy slots');
  const tied=layout([['in','c','hub'],['self','hub','hub'],['out','hub','a']]),self=tied.edges.find(row=>row.id==='self'),other=tied.edges.find(row=>row.id==='out');
  assert.equal(self.label.y+self.label.h/2,other.label.y+other.label.h/2,'upper source labels share an equal row');
  assert.deepEqual(left(tied),[['in','target',140.59338662244784],['self','source',110.34338662244782],['out','source',122.84338662244782]],'stable partition keeps native source ordinal, even for equal label rows');
  assert.equal(self.points[0][0],node(tied).x);assert.equal(self.points.at(-1)[0],node(tied).x+182);assert.equal(self.points.at(-1)[1],128.09338662244784,'self target remains distinct from its adjusted source');
  console.log('Exact legacy zero-role/multiple-target/height/final-direction fallbacks; equal native-order ties and distinct self endpoints PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn native_scope_controls_expand_without_post_and_keep_complete_table_and_pending_refusal() {
    check_controller(
        r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  const base=structuredClone(modelDocument.entities[0]);modelDocument.entities=Array.from({length:52},(_,i)=>({...structuredClone(base),id:`ety_scale_${i}`,display_name:i?'Type '+i:'직원 Employee 🧭'}));
  modelDocument.links=Array.from({length:148},(_,i)=>({id:`lty_scale_${i}`,from_entity_type:'ety_scale_0',to_entity_type:`ety_scale_${i%51+1}`,cardinality:'one_to_many',allow_cross_tenant:false,revision:1,display:null}));
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  const scene=find('live-ontology-scene'),cards=()=>scene.querySelectorAll('[data-model-type]');
  assert.equal(find('live-ontology-scope-related').getAttribute('aria-pressed'),'true');assert.ok(cards().length<52 && cards().length>0);assert.equal(scene.style.transform,'scale(1)','initial framing retains natural text size');
  assert.equal(find('live-ontology-model-table').querySelectorAll('[data-model-type]').length,52);assert.equal(find('live-ontology-model-table').querySelectorAll('[data-model-link]').length,148);
  const before=cards().length,requests=calls.length;find('live-ontology-expand').emit('click');assert.ok(cards().length>before);assert.equal(calls.length,requests);
  find('live-ontology-scope-all').emit('click');assert.equal(cards().length,52);assert.equal(scene.querySelectorAll('[data-model-edge]').length,148);
  find('live-ontology-overview').emit('click');assert.match(find('live-ontology-graph-frame').textContent,/Overview.*overflows/);find('live-ontology-fit').emit('click');assert.equal(scene.style.transform,'scale(1)');
  find('live-ontology-scope-selected').emit('click');assert.equal(cards().length,1);find('live-ontology-search').value='absent';find('live-ontology-search').emit('input');assert.equal(cards().length,1);assert.match(find('live-ontology-graph-counts').textContent,/remaining.*outside scope.*undrawn/);
  assert.ok(calls.slice(requests).every(([,options])=>(options.method||'GET')==='GET'),'all navigation is zero-write');
  console.log('Native Selected/Related/All, deliberate expansion, overview disclosure, readable selection, complete Table and no navigationPOST PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}
