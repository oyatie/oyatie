pub(super) const SCRIPT: &str = r#"
function createOntologyValueForm(control, host, changed, label, catalog = () => []) {
  host.setAttribute('role','group');host.setAttribute('aria-label',`${label} value declaration`);
  const scalars=['string','integer','double','boolean','date','timestamp'];let root=null,enabled=false,original=null,originalState=null;
  const initial=value=>value===null?{kind:'legacy'}:value.shape==='scalar'?{kind:value.kind}:value.shape==='named_string'?{kind:'named_string',reference:{shape:'named_string',type_id:value.type_id,revision:value.revision}}:value.shape==='array'?{kind:'array',element:initial(value.element)}:{kind:'struct',fields:value.fields.map(f=>({name:f.name,presence:f.required?'required':'optional',value:initial(f.value_type)}))};
  const scalar=()=>({kind:'string'}),kinds=[...scalars.map(s=>[s,s[0].toUpperCase()+s.slice(1)]),['array','Array'],['struct','Struct']];
  const make=(tag,label)=>{const element=document.createElement(tag);element.setAttribute('aria-label',label);return element;};
  const labeled=(text,control)=>{const label=document.createElement('label');label.textContent=text;label.append(control);return label;};
  function choices(select,values){select.replaceChildren(...values.map(([value,label])=>{const option=document.createElement('option');option.value=value;option.textContent=label;return option;}));}
  function node(select,nested,state,depth,label,top=false) {
    let shown=null,element=null,fields=[],fieldNumber=0,reference=null,refreshReference=()=>{};
    if(state.kind==='named_string' && !Array.from(select.children).some(o=>o.value==='named_string')){const option=document.createElement('option');option.value='named_string';option.textContent='Loaded unsupported nested reusable type';option.disabled=true;select.append(option);}
    select.value=state.kind;
    function render(next) {
      shown=select.value;nested.replaceChildren();element=null;fields=[];reference=null;refreshReference=()=>{};
      if(shown==='named_string') {
        const picker=make('select',`${label} exact reusable type`),retained=next.reference;
        let catalogState=null;
        refreshReference=()=>{
          const rows=catalog(),unique=rows.filter(row=>rows.filter(other=>other.id===row.id && other.revision===row.revision).length===1);
          const entries=unique.map(row=>[JSON.stringify({shape:'named_string',type_id:row.id,revision:row.revision}),`${row.display_name} · ${row.id} · revision ${row.revision}`]),state=JSON.stringify(entries);if(state===catalogState)return;
          const key=catalogState===null?(retained===undefined?undefined:retained?JSON.stringify(retained):''):picker.value;catalogState=state;
          if(key!==undefined && !entries.some(([value])=>value===key)){if(key){const exact=JSON.parse(key);entries.push([key,`Unresolved or ambiguous · ${exact.type_id} · revision ${exact.revision}`]);}else entries.unshift(['','Choose an exact reusable type']);}
          choices(picker,entries);if(key!==undefined)picker.value=key;
        };
        refreshReference();
        picker.dataset.nestedReference=String(!top);picker.addEventListener('input',()=>changed());nested.append(labeled('Exact reusable type',picker));reference=()=>picker.value?JSON.parse(picker.value):null;
        if(!top){const warning=document.createElement('p');warning.textContent='Loaded nested reusable reference is preserved. Native validation refuses this position; choose a supported nested type to repair it.';nested.append(warning);}
      } else if(shown==='array') {
        const row=make('fieldset',`${label} array element`),legend=document.createElement('legend');legend.textContent='Array element';
        const child=make('select',`${label} array element type`),body=document.createElement('div');choices(child,kinds);row.append(legend,labeled('Type',child),body);nested.append(row);
        element=node(child,body,next.element || scalar(),depth+1,`${label} element`);
      } else if(shown==='struct') {
        const rows=document.createElement('div');nested.append(rows);
        function add(value) {
          const fieldLabel=`${label} field ${++fieldNumber}`,row=make('fieldset',fieldLabel),legend=document.createElement('legend');legend.textContent=`Field ${fieldNumber}`;
          const name=make('input',`${fieldLabel} name`);name.type='text';name.maxLength=4096;name.value=value.name;
          const presence=make('select',`${fieldLabel} presence`);choices(presence,[['required','Required'],['optional','Optional']]);presence.value=value.presence;
          const kind=make('select',`${fieldLabel} value type`);choices(kind,kinds);
          const body=document.createElement('div'),remove=make('button',`Remove ${fieldLabel}`);remove.type='button';remove.textContent='Remove field';row.append(legend,labeled('Name',name),labeled('Required',presence),labeled('Type',kind),body,remove);rows.append(row);
          const field={row,name,presence,value:node(kind,body,value.value,depth+1,fieldLabel)};fields.push(field);
          for(const input of [name,presence])input.addEventListener('input',()=>changed());
          remove.addEventListener('click',()=>{
            if(!enabled)return;
            const index=fields.indexOf(field),focused=document.activeElement,owned=row.contains(focused);
            fields=fields.filter(f=>f!==field);rows.removeChild(row);changed();
            const target=fields[index]?.name || button;
            if(owned && (document.activeElement===document.body || document.activeElement===focused && !focused.isConnected) && target.isConnected && !target.disabled && target.getClientRects().length)target.focus();
          });
        }
        for(const field of next.fields || [])add(field);
        const button=make('button',`Add ${label} struct field`);button.type='button';button.textContent='Add field';button.dataset.depthLimit=String(depth>=8);
        button.addEventListener('click',()=>{if(!enabled || depth>=8 || fields.length>=256)return;add({name:'',presence:'required',value:scalar()});setEnabled(enabled);changed();});nested.append(button);
      }
      setEnabled(enabled);
    }
    function sync(){if(select.value!==shown)render({kind:select.value});}
    if(!top)select.addEventListener('change',()=>{sync();changed();});
    render(state);
    return {
      refresh(){sync();refreshReference();element?.refresh();for(const field of fields)field.value.refresh();},
      capture(){sync();return {kind:select.value,...(shown==='named_string'?{reference:reference()}:shown==='array'?{element:element.capture()}:shown==='struct'?{fields:fields.map(f=>({name:f.name.value,presence:f.presence.value,value:f.value.capture()}))}:{})};},
      read(budget){
        sync();if(depth>8 || --budget.nodes<0)throw new Error('Value declarations allow at most 8 levels and 4096 nodes.');
        if(shown==='named_string'){const value=reference();if(!value)throw new Error('Choose an exact reusable type from the model catalog.');if(top && catalog().filter(row=>row.id===value.type_id && row.revision===value.revision).length!==1)throw new Error('Selected reusable type is unresolved or ambiguous in the current model catalog. Choose an exact reusable type.');return value;}
        if(scalars.includes(shown))return {shape:'scalar',kind:shown};
        if(shown==='legacy' && top)return null;
        if(shown==='array')return {shape:'array',element:element.read(budget)};
        if(shown!=='struct' || !fields.length || fields.length>256)throw new Error('Choose a supported native value declaration; a struct needs 1 to 256 named fields.');
        const names=new Set();return {shape:'struct',fields:fields.map(f=>{
          const name=f.name.value;if(/^\p{White_Space}*$/u.test(name) || new TextEncoder().encode(name).length>4096 || names.has(name))throw new Error('Struct fields need unique names of at most 4096 bytes.');names.add(name);
          if(!['required','optional'].includes(f.presence.value))throw new Error('Choose Required or Optional presence for each struct field.');
          return {name,required:f.presence.value==='required',value_type:f.value.read(budget)};
        })};
      },
    };
  }
  function setEnabled(value){enabled=Boolean(value);for(const tag of ['input','textarea','select','button'])for(const child of host.querySelectorAll(tag))child.disabled=!enabled || child.dataset.depthLimit==='true' || child.dataset.nestedReference==='true';}
  control.addEventListener('change',()=>{root?.capture();changed();});
  return {
    write(value){original=value;root=node(control,host,initial(value),1,label,true);originalState=root.capture();},
    read(){if(!root)throw new Error('Choose a native value declaration.');if(ontologyModelEqual(root.capture(),originalState))return original;return root.read({nodes:4096});},
    capture(){let focus=null;for(const tag of ['input','select','button']){const index=Array.from(host.querySelectorAll(tag)).indexOf(document.activeElement);if(index>=0)focus={tag,index};}return {value:root?.capture() || scalar(),original,originalState,focus};},
    restore(snapshot){original=snapshot.original;originalState=snapshot.originalState;root=node(control,host,snapshot.value,1,label,true);if(snapshot.focus && document.activeElement===document.body)host.querySelectorAll(snapshot.focus.tag)[snapshot.focus.index]?.focus({preventScroll:true});},
    enable(value){root?.refresh();setEnabled(value);},
    clear(){host.replaceChildren();root=null;original=null;originalState=null;},
  };
}
"#;
