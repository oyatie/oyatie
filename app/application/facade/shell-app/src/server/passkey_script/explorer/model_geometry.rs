pub(super) const SCRIPT: &str = r#"
function ontologyGraphPlan(types, links, selected, sizes, viewport, mode, retained=null) {
  const positive=value=>Number.isFinite(value) && value>0;
  if(!positive(viewport.width) || !positive(viewport.height) || !positive(viewport.padding) || types.some(row=>!positive(sizes.types.get(row.entity_type)?.w) || !positive(sizes.types.get(row.entity_type)?.h)) || links.some(row=>!positive(sizes.links.get(row.link_type)?.w) || !positive(sizes.links.get(row.link_type)?.h)))return null;
  const p=viewport.padding,margin=p+Math.hypot(7,4)*1.5,byId=new Map(types.map(row=>[row.entity_type,row]));
  const relationship=links.find(row=>row.link_type===selected.link);
  const primary=byId.has(selected.type)?selected.type:relationship && byId.has(relationship.from_entity_type)?relationship.from_entity_type:types[0]?.entity_type;
  if(!primary)return {width:2*margin,height:2*margin,nodes:[],edges:[],neighbors:0,totalNeighbors:0};
  const forced=relationship && relationship.to_entity_type!==primary && byId.has(relationship.to_entity_type)?[relationship.to_entity_type]:[];
  const neighbors=[...forced,...types.map(row=>row.entity_type).filter(id=>id!==primary && !forced.includes(id)).sort()];
  const cardW=Math.max(...types.map(row=>sizes.types.get(row.entity_type).w));
  const direct=links.filter(row=>row.from_entity_type!==row.to_entity_type && [row.from_entity_type,row.to_entity_type].includes(primary));
  const corridor=Math.max(44,...direct.map(row=>sizes.links.get(row.link_type).w))+2*p;
  const maximumColumns=3*cardW+2*corridor+2*margin<=viewport.width?3:2*cardW+corridor+2*margin<=viewport.width?2:1;
  const stack=rows=>rows.reduce((height,row)=>height+sizes.links.get(row.link_type).h+p/2,0)-(rows.length?p/2:0);
  const self=id=>links.filter(row=>row.from_entity_type===id && row.to_entity_type===id);
  const groupHeight=id=>sizes.types.get(id).h+stack(self(id))+(self(id).length?p:0);
  const connections=id=>direct.filter(row=>row.from_entity_type===id || row.to_entity_type===id);
  function plan(count) {
    const shown=neighbors.slice(0,count),visible=new Set([primary,...shown]);
    const visibleLinks=links.filter(row=>visible.has(row.from_entity_type) && visible.has(row.to_entity_type));
    const induced=visibleLinks.filter(row=>row.from_entity_type!==row.to_entity_type && ![row.from_entity_type,row.to_entity_type].includes(primary));
    let columns=maximumColumns,clearance,margin,corridor,ends,ports,lanes;
    while(true){
      const column=new Map([[primary,columns===3?1:0]]);
      shown.forEach((id,index)=>column.set(id,columns===1?0:columns===3?index%2*2:1));
      ends=new Map();ports=new Map();lanes=new Map();
      const endpoint=(id,side)=>{
        const point={id,side,portKey:JSON.stringify([id,side]),laneKey:JSON.stringify([column.get(id),side])};
        for(const[group,key]of [[ports,point.portKey],[lanes,point.laneKey]]){if(!group.has(key))group.set(key,[]);group.get(key).push(point);}
        return point;
      };
      for(const row of visibleLinks){
        const a=row.from_entity_type,b=row.to_entity_type,directLink=[a,b].includes(primary);
        let fromSide,toSide;
        if(a===b){fromSide='left';toSide='right';}
        else if(directLink && column.get(a)!==column.get(b)){fromSide=column.get(a)<column.get(b)?'right':'left';toSide=fromSide==='left'?'right':'left';}
        else{const first=directLink?primary:column.get(a)>column.get(b)?b:a;fromSide=first===a?'left':'right';toSide=first===a?'right':'left';}
        ends.set(row.link_type,[endpoint(a,fromSide),endpoint(b,toSide)]);
      }
      // Reserve two scene pixels per competing lane without shrinking native targets.
      clearance=Math.max(p,2*(Math.max(0,...[...lanes.values()].map(rows=>rows.length))+1));
      margin=clearance+Math.hypot(7,4)*1.5;corridor=Math.max(44,...direct.map(row=>sizes.links.get(row.link_type).w))+2*clearance;
      if(columns===1 || columns*cardW+(columns-1)*corridor+2*margin<=viewport.width)break;
      columns--;
    }
    const rowHeight=id=>columns===1?groupHeight(id)+stack(connections(id))+(connections(id).length?p:0):Math.max(groupHeight(id),stack(connections(id)));
    const rails=Array.from({length:columns===3?2:1},()=>[]);
    shown.forEach((id,index)=>rails[index%rails.length].push(id));
    const railHeights=rails.map(rows=>rows.reduce((height,id)=>height+rowHeight(id)+p,0)-(rows.length?p:0));
    const baseHeight=groupHeight(primary),body=columns===1?baseHeight+(shown.length?p:0)+railHeights[0]:Math.max(baseHeight,...railHeights);
    const width=columns*cardW+(columns-1)*corridor+2*margin,height=2*margin+body+induced.reduce((sum,row)=>sum+sizes.links.get(row.link_type).h+p,0);
    const positions=new Map(),labelPositions=new Map();
    function place(id,x,y) {
      for(const row of self(id)){const size=sizes.links.get(row.link_type);labelPositions.set(row.link_type,{x:x+(cardW-size.w)/2,y});y+=size.h+p/2;}
      if(self(id).length)y+=p/2;
      const size=sizes.types.get(id);positions.set(id,{id,x,y,w:size.w,h:size.h});
    }
    place(primary,margin+(columns===3?1:0)*(cardW+corridor),margin+(columns===1?0:(body-baseHeight)/2));
    rails.forEach((rows,index)=>{
      let y=columns===1?margin+baseHeight+(shown.length?p:0):margin+(body-railHeights[index])/2;
      for(const id of rows){
        const rows=connections(id),labels=stack(rows),row=rowHeight(id),x=margin+(columns===1?0:columns===3?index===0?0:2:1)*(cardW+corridor);
        let labelY=columns===1?y:y+(row-labels)/2;
        for(const link of rows){const size=sizes.links.get(link.link_type),left=columns===1?margin+(cardW-size.w)/2:Math.min(x,positions.get(primary).x)+cardW+(corridor-size.w)/2;labelPositions.set(link.link_type,{x:left,y:labelY});labelY+=size.h+p/2;}
        place(id,x,columns===1?y+labels+(rows.length?p:0):y+(row-groupHeight(id))/2);y+=row+p;
      }
    });
    let footer=margin+body;
    for(const row of induced){const size=sizes.links.get(row.link_type),a=positions.get(row.from_entity_type),b=positions.get(row.to_entity_type),left=Math.min(a.x,b.x)-p/2,right=Math.max(a.x+a.w,b.x+b.w)+p/2;footer+=p;labelPositions.set(row.link_type,{x:(left+right-size.w)/2,y:footer});footer+=size.h;}
    const endpoint=point=>{
      const node=positions.get(point.id),incident=ports.get(point.portKey),competing=lanes.get(point.laneKey);
      const ratio=(incident.indexOf(point)+1)/(incident.length+1),lane=clearance*(competing.indexOf(point)+1)/(competing.length+1);
      return {x:node.x+(point.side==='left'?0:node.w),y:node.y+p/2+Math.max(0,node.h-p)*ratio,lane:node.x+(point.side==='left'?-lane:cardW+lane)};
    };
    const edges=visibleLinks.map(row=>{
      const label=labelPositions.get(row.link_type),size=sizes.links.get(row.link_type),[from,to]=ends.get(row.link_type).map(endpoint),y=label.y+size.h/2;
      const points=[[from.x,from.y],[from.lane,from.y],[from.lane,y],[to.lane,y],[to.lane,to.y],[to.x,to.y]];
      return {id:row.link_type,from:row.from_entity_type,to:row.to_entity_type,label:{...label,w:size.w,h:size.h},points,path:points.map(([x,y],index)=>`${index?'L':'M'} ${x} ${y}`).join(' ')};
    });
    return {width,height,nodes:[...positions.values()],edges,neighbors:count,totalNeighbors:neighbors.length,columns};
  }
  if(mode!=='related')return plan(neighbors.length);
  let count=forced.length;
  for(let next=count+1;next<=neighbors.length;next++){const candidate=plan(next);if(candidate.width>viewport.width || candidate.height>viewport.height)break;count=next;}
  return plan(Math.min(neighbors.length,Math.max(count,retained ?? 0)));
}
"#;
