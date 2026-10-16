pub(super) const SCRIPT: &str = r#"
function ontologyDefaultConstant(value, budget={nodes:4096}, depth=1) {
  const object=v=>v!==null && typeof v==='object' && !Array.isArray(v),shape=(v,keys)=>object(v) && Object.keys(v).length===keys.length && keys.every(key=>Object.hasOwn(v,key));
  const text=v=>typeof v==='string' && new TextEncoder().encode(v).length<=4096;
  const integer=v=>typeof v==='string' && /^(0|-?[1-9][0-9]*)$/.test(v) && v.length<=20 && BigInt(v)>=-9223372036854775808n && BigInt(v)<=9223372036854775807n;
  if(depth>8 || --budget.nodes<0 || !object(value))return false;
  if(value.kind==='integer')return shape(value,['kind','value']) && integer(value.value);
  if(value.kind==='timestamp')return shape(value,['kind','epoch_millis']) && integer(value.epoch_millis);
  if(value.kind==='string')return shape(value,['kind','value']) && text(value.value);
  if(value.kind==='double')return shape(value,['kind','value']) && typeof value.value==='number' && Number.isFinite(value.value);
  if(value.kind==='boolean')return shape(value,['kind','value']) && typeof value.value==='boolean';
  if(value.kind==='date'){
    if(!shape(value,['kind','year','month','day']) || !Number.isInteger(value.year) || value.year < -2147483648 || value.year > 2147483647 || !Number.isInteger(value.month) || value.month<1 || value.month>12 || !Number.isInteger(value.day))return false;
    const leap=value.year%4===0 && (value.year%100!==0 || value.year%400===0),days=[31,leap?29:28,31,30,31,30,31,31,30,31,30,31];return value.day>=1 && value.day<=days[value.month-1];
  }
  if(value.kind==='array')return shape(value,['kind','items']) && Array.isArray(value.items) && value.items.length<=256 && value.items.every(item=>ontologyDefaultConstant(item,budget,depth+1));
  return value.kind==='struct' && shape(value,['kind','fields']) && Array.isArray(value.fields) && value.fields.length<=256 && new Set(value.fields.map(field=>field?.name)).size===value.fields.length && value.fields.every(field=>shape(field,['name','value']) && text(field.name) && ontologyDefaultConstant(field.value,budget,depth+1));
}
function ontologyDefaultText(value){
  if(value===undefined)return 'No default';
  if(value.kind==='array')return `[${value.items.map(ontologyDefaultText).join(', ')}]`;
  if(value.kind==='struct')return `{${value.fields.map(field=>`${field.name}: ${ontologyDefaultText(field.value)}`).join(', ')}}`;
  if(value.kind==='date')return `${value.year}-${String(value.month).padStart(2,'0')}-${String(value.day).padStart(2,'0')}`;
  if(value.kind==='timestamp')return `${value.epoch_millis} ms since Unix epoch`;
  return value.kind==='string'?JSON.stringify(value.value):String(value.value);
}
function createOntologyDefaultForm(host, changed, label, catalog=()=>[], readType=null) {
  let declaration=null,present=false,root=null,enabled=false,original,originalState=null,problem='';
  const currentType=()=>readType?readType():declaration;
  const controls=()=>['input','textarea','select','button'].flatMap(tag=>Array.from(host.querySelectorAll(tag)));
  const make=(tag,name)=>{const node=document.createElement(tag);node.setAttribute('aria-label',name);return node;};
  const labeled=(name,node)=>{const label=document.createElement('label');label.textContent=name;label.append(node);return label;};
  const button=(text,action)=>{const node=make('button',`${label} ${text.toLowerCase()}`);node.type='button';node.textContent=text;node.addEventListener('click',()=>{if(enabled)try{action();}catch(problem){error(problem.message);}});return node;};
  const kind=type=>type===null || type?.shape==='named_string'?'string':type?.shape==='scalar'?type.kind:type?.shape;
  function initial(type,depth=1){
    if(depth>8)throw new Error('Defaults allow at most 8 levels.');
    const name=kind(type);
    if(type?.shape==='named_string'){const rows=catalog().filter(row=>row.id===type.type_id && row.revision===type.revision);return {kind:'string',value:rows.length===1?rows[0].members[0] ?? '':''};}
    if(name==='array')return {kind:name,items:[]};
    if(name==='struct')return {kind:name,fields:type.fields.filter(field=>field.required).map(field=>({name:field.name,value:initial(field.value_type,depth+1)}))};
    return name==='date'?{kind:name,year:1970,month:1,day:1}:name==='timestamp'?{kind:name,epoch_millis:'0'}:{kind:name,value:name==='boolean'?false:name==='double'?0:name==='integer'?'0':''};
  }
  function node(type,value,name,depth=1){
    const host=make('div',`${name} default value`),inputs=[];
    const field=(text,raw,inputType='text',tag='input')=>{const control=make(tag,text);control.type=inputType;control.value=String(raw);control.maxLength=4096;control.addEventListener('input',()=>changed());inputs.push(control);host.append(labeled(text,control));return control;};
    let read,capture;
    if(value.kind==='array'){
      const rows=document.createElement('div');host.append(rows);let items=[],serial=0;
      function add(item){
        const itemName=`${name} item ${++serial}`,entry=node(type?.shape==='array'?type.element:null,item,itemName,depth+1),row=document.createElement('fieldset'),legend=document.createElement('legend');legend.textContent=itemName;
        const record={entry,row};row.append(legend,entry.host,button(`Remove ${itemName}`,()=>{const owned=row.contains(document.activeElement),index=items.indexOf(record);items=items.filter(item=>item!==record);rows.removeChild(row);changed();enable(enabled);if(owned)(items[index]?.entry.target || addButton).focus({preventScroll:true});}));rows.append(row);items.push(record);
      }
      value.items.forEach(add);
      const addButton=button(`Add ${name} item`,()=>{if(items.length>=256 || depth>=8)return;add(initial(type?.shape==='array'?type.element:null,depth+1));enable(enabled);changed();});addButton.dataset.depthLimit=String(depth>=8);host.append(addButton);
      read=()=>({kind:'array',items:items.map(item=>item.entry.read())});capture=()=>({kind:'array',items:items.map(item=>item.entry.capture())});
    }else if(value.kind==='struct'){
      const rows=document.createElement('div');host.append(rows);let fields=[];
      const declared=type?.shape==='struct'?type.fields:[];
      function add(fieldName,value){
        const declaration=declared.find(field=>field.name===fieldName),entry=node(declaration?.value_type ?? null,value,`${name} field ${fieldName}`,depth+1),row=document.createElement('fieldset'),legend=document.createElement('legend');legend.textContent=fieldName;
        const record={name:fieldName,entry,row};row.append(legend,entry.host,button(`Remove ${name} field ${fieldName}`,()=>{const owned=row.contains(document.activeElement);fields=fields.filter(field=>field!==record);rows.removeChild(row);refresh();changed();if(owned)target.focus({preventScroll:true});}));rows.append(row);fields.push(record);
      }
      value.fields.forEach(field=>add(field.name,field.value));
      const additions=document.createElement('div');host.append(additions);
      function refresh(){additions.replaceChildren(...declared.filter(field=>!fields.some(row=>row.name===field.name)).map(field=>{const addButton=button(`Add ${name} field ${field.name}`,()=>{if(fields.length>=256 || depth>=8)return;add(field.name,initial(field.value_type,depth+1));refresh();changed();});addButton.dataset.depthLimit=String(depth>=8);return addButton;}));enable(enabled);}
      const target=host;refresh();
      read=()=>({kind:'struct',fields:fields.map(field=>({name:field.name,value:field.entry.read()}))});capture=()=>({kind:'struct',fields:fields.map(field=>({name:field.name,value:field.entry.capture()}))});
    }else if(value.kind==='date'){
      const year=field(`${name} year`,value.year),month=field(`${name} month`,value.month),day=field(`${name} day`,value.day);
      read=()=>{if(!/^-?(0|[1-9][0-9]*)$/.test(year.value) || !/^[1-9][0-9]*$/.test(month.value) || !/^[1-9][0-9]*$/.test(day.value))throw new Error('Enter a valid year, month and day.');return {kind:'date',year:Number(year.value),month:Number(month.value),day:Number(day.value)};};capture=()=>({kind:'date',year:year.value,month:month.value,day:day.value});
    }else if(value.kind==='boolean' || type?.shape==='named_string' && value.kind==='string'){
      const select=make('select',`${name} default value`),matches=catalog().filter(row=>row.id===type?.type_id && row.revision===type?.revision);
      const values=value.kind==='boolean'?['false','true']:matches.length===1?[...matches[0].members]:[];
      if(!values.includes(String(value.value)))values.unshift(String(value.value));
      select.replaceChildren(...values.map(value=>{const option=document.createElement('option');option.value=value;option.textContent=value;return option;}));select.value=String(value.value);select.addEventListener('input',()=>changed());inputs.push(select);host.append(labeled(`${name} default value`,select));
      read=()=>{if(value.kind==='boolean' && !['true','false'].includes(select.value))throw new Error('Choose True or False.');return {kind:value.kind,value:value.kind==='boolean'?select.value==='true':select.value};};capture=()=>({kind:value.kind,value:select.value});
    }else{
      const input=field(value.kind==='timestamp'?`${name} timestamp (milliseconds since Unix epoch)`:`${name} default value`,value.kind==='timestamp'?value.epoch_millis:value.value,'text',value.kind==='string'?'textarea':'input');
      if(['integer','double','timestamp'].includes(value.kind))input.inputMode=value.kind==='double'?'decimal':'numeric';
      read=()=>{if(value.kind==='double' && !/^[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?$/.test(input.value))throw new Error('Enter a finite decimal number.');return value.kind==='timestamp'?{kind:value.kind,epoch_millis:input.value}:{kind:value.kind,value:value.kind==='double'?Number(input.value):input.value};};capture=()=>value.kind==='timestamp'?{kind:value.kind,epoch_millis:input.value}:{kind:value.kind,value:input.value};
    }
    const target=inputs[0] || host;if(!inputs.length)target.setAttribute('tabindex','-1');
    return {host,target,read,capture};
  }
  function enable(value){enabled=Boolean(value);for(const control of controls())control.disabled=!enabled || control.dataset.depthLimit==='true';if(root)root.target.disabled=!enabled;}
  function error(message){problem=message || '';const target=root?.target,output=document.getElementById(`${host.id}-error`);if(output){output.textContent=problem;output.hidden=!problem;}if(target){target.setAttribute('aria-invalid',String(Boolean(problem)));target.setAttribute('aria-describedby',`${host.id}-help${problem?` ${host.id}-error`:''}`);}}
  function render(value){
    host.replaceChildren();host.setAttribute('role','group');host.setAttribute('aria-label',`${label} default`);root=null;
    const help=document.createElement('p');help.id=`${host.id}-help`;help.textContent='Used when a new object omits this property. Explicit values are kept. Existing objects are unchanged.';host.append(help);
    if(!present)host.append(button('Add default',()=>{declaration=currentType();present=true;render(initial(declaration));changed();root.target.focus({preventScroll:true});}));
    else{
      root=node(declaration,value,label);root.target.id=`${host.id}-value`;host.append(root.host);
      if(value.kind!==kind(declaration))host.append(button('Use declared value type',()=>{declaration=currentType();render(initial(declaration));changed();root.target.focus({preventScroll:true});}));
      host.append(button('Remove default',()=>{present=false;render();changed();host.querySelectorAll('button')[0]?.focus({preventScroll:true});}));
    }
    const output=document.createElement('p');output.id=`${host.id}-error`;output.setAttribute('class','foundry-default-error');output.setAttribute('role','status');host.append(output);enable(enabled);error(problem);
  }
  return {
    write(type,value){declaration=type;present=value!==undefined;original=value;problem='';render(value);originalState=root?.capture() ?? null;},
    read(){if(!present)return undefined;try{const result=ontologyModelEqual(root.capture(),originalState)?original:root.read();if(!ontologyDefaultConstant(result)){error('Enter a valid default value within the native type and size limits.');throw new Error('Enter a valid default value within the native type and size limits.');}return result;}catch(problem){error(problem.message);throw problem;}},
    capture(){return {declaration,present,original,originalState,value:root?.capture(),problem,focus:controls().indexOf(document.activeElement)};},
    restore(snapshot){declaration=snapshot.declaration;present=snapshot.present;original=snapshot.original;originalState=snapshot.originalState;problem=snapshot.problem;render(snapshot.value);if(snapshot.focus>=0 && document.activeElement===document.body)controls()[snapshot.focus]?.focus({preventScroll:true});},
    enable,error,clear(){declaration=null;present=false;original=undefined;originalState=null;problem='';host.replaceChildren();},
  };

}
"#;
