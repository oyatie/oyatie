pub(super) const SCRIPT: &str = r#"
function createOntologyValueForm(control, host, changed) {
  const scalars=['string','integer','double','boolean','date','timestamp'];let root=null,enabled=false;
  const initial=value=>value===null?{kind:'legacy'}:value.shape==='scalar'?{kind:value.kind}:value.shape==='array'?{kind:'array',element:initial(value.element)}:{kind:'struct',fields:value.fields.map(f=>({name:f.name,presence:f.required?'required':'optional',value:initial(f.value_type)}))};
  const scalar=()=>({kind:'string'}),kinds=[...scalars.map(s=>[s,s[0].toUpperCase()+s.slice(1)]),['array','Array'],['struct','Struct']];
  const make=(tag,label)=>{const element=document.createElement(tag);element.setAttribute('aria-label',label);return element;};
  function choices(select,values){select.replaceChildren(...values.map(([value,label])=>{const option=document.createElement('option');option.value=value;option.textContent=label;return option;}));}
  function node(select,nested,state,depth,label,top=false) {
    let shown=null,element=null,fields=[],fieldNumber=0;select.value=state.kind;
    function render(next) {
      shown=select.value;nested.replaceChildren();element=null;fields=[];
      if(shown==='array') {
        const row=make('fieldset',`${label} array element`),legend=document.createElement('legend');legend.textContent='Array element';
        const child=make('select',`${label} array element type`),body=document.createElement('div');choices(child,kinds);row.append(legend,child,body);nested.append(row);
        element=node(child,body,next.element || scalar(),depth+1,`${label} element`);
      } else if(shown==='struct') {
        const rows=document.createElement('div');nested.append(rows);
        function add(value) {
          const fieldLabel=`${label} field ${++fieldNumber}`,row=make('fieldset',fieldLabel),legend=document.createElement('legend');legend.textContent=`Field ${fieldNumber}`;
          const name=make('input',`${fieldLabel} name`);name.type='text';name.maxLength=4096;name.value=value.name;
          const presence=make('select',`${fieldLabel} presence`);choices(presence,[['required','Required'],['optional','Optional']]);presence.value=value.presence;
          const kind=make('select',`${fieldLabel} value type`);choices(kind,kinds);
          const body=document.createElement('div'),remove=make('button',`Remove ${fieldLabel}`);remove.type='button';remove.textContent='Remove field';row.append(legend,name,presence,kind,body,remove);rows.append(row);
          const field={row,name,presence,value:node(kind,body,value.value,depth+1,fieldLabel)};fields.push(field);
          for(const input of [name,presence])input.addEventListener('input',()=>changed());
          remove.addEventListener('click',()=>{if(!enabled)return;fields=fields.filter(f=>f!==field);rows.replaceChildren(...fields.map(f=>f.row));changed();});
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
      capture(){sync();return {kind:select.value,...(shown==='array'?{element:element.capture()}:shown==='struct'?{fields:fields.map(f=>({name:f.name.value,presence:f.presence.value,value:f.value.capture()}))}:{})};},
      read(budget){
        sync();if(depth>8 || --budget.nodes<0)throw new Error('Value declarations allow at most 8 levels and 4096 nodes.');
        if(scalars.includes(shown))return {shape:'scalar',kind:shown};
        if(shown==='legacy' && top)return null;
        if(shown==='array')return {shape:'array',element:element.read(budget)};
        if(shown!=='struct' || !fields.length || fields.length>256)throw new Error('Choose a supported native value declaration; a struct needs 1 to 256 named fields.');
        const names=new Set();return {shape:'struct',fields:fields.map(f=>{
          const name=f.name.value;if(!name.trim() || new TextEncoder().encode(name).length>4096 || names.has(name))throw new Error('Struct fields need unique names of at most 4096 bytes.');names.add(name);
          if(!['required','optional'].includes(f.presence.value))throw new Error('Choose Required or Optional presence for each struct field.');
          return {name,required:f.presence.value==='required',value_type:f.value.read(budget)};
        })};
      },
    };
  }
  function setEnabled(value){enabled=Boolean(value);for(const tag of ['input','select','button'])for(const child of host.querySelectorAll(tag))child.disabled=!enabled || child.dataset.depthLimit==='true';}
  control.addEventListener('change',()=>{root?.capture();changed();});
  return {
    write(value){root=node(control,host,initial(value),1,'Property',true);},
    read(){if(!root)throw new Error('Choose a native value declaration.');return root.read({nodes:4096});},
    capture(){let focus=null;for(const tag of ['input','select','button']){const index=Array.from(host.querySelectorAll(tag)).indexOf(document.activeElement);if(index>=0)focus={tag,index};}return {value:root?.capture() || scalar(),focus};},
    restore(snapshot){root=node(control,host,snapshot.value,1,'Property',true);if(snapshot.focus && document.activeElement===document.body)host.querySelectorAll(snapshot.focus.tag)[snapshot.focus.index]?.focus({preventScroll:true});},
    enable:setEnabled,
    clear(){host.replaceChildren();root=null;},
  };
}
"#;
