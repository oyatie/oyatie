pub(super) const SCRIPT: &str = r#"
function createOntologyMemberForm(host,changed,label) {
  host.setAttribute('role','group');host.setAttribute('aria-label',`${label} allowed values`);
  let rows=[],number=0,enabled=false;
  const list=document.createElement('div'),add=document.createElement('button');add.type='button';add.textContent='Add allowed value';add.setAttribute('aria-label',`Add ${label} allowed value`);host.append(list,add);
  function append(value){
    const index=++number,row=document.createElement('div'),caption=document.createElement('label'),input=document.createElement('textarea'),remove=document.createElement('button');
    row.className='foundry-value-member';caption.textContent=`Allowed value ${index}`;input.rows=2;input.setAttribute('aria-label',`${label} allowed value ${index}`);input.value=value;caption.append(input);
    remove.type='button';remove.textContent='Remove value';remove.setAttribute('aria-label',`Remove ${label} allowed value ${index}`);row.append(caption,remove);list.append(row);
    const member={row,input,value};rows.push(member);
    input.addEventListener('input',()=>{member.value=input.value;changed();});
    remove.addEventListener('click',()=>{
      if(!enabled)return;const index=rows.indexOf(member),focused=document.activeElement,owned=row.contains(focused);
      rows=rows.filter(other=>other!==member);list.removeChild(row);enable(enabled);changed();
      const next=rows[index]?.input || add;
      if(owned && (document.activeElement===document.body || document.activeElement===focused && !focused.isConnected) && next.isConnected && !next.disabled && next.getClientRects().length)next.focus({preventScroll:true});
    });
  }
  function enable(value){enabled=Boolean(value);add.disabled=!enabled || rows.length>=256;for(const row of rows){row.input.disabled=!enabled;for(const button of row.row.querySelectorAll('button'))button.disabled=!enabled;}}
  function write(members){list.replaceChildren();rows=[];number=0;for(const value of members)append(value);enable(enabled);}
  add.addEventListener('click',()=>{if(!enabled || rows.length>=256)return;append('');enable(enabled);changed();const input=rows.at(-1).input;if(input.isConnected && !input.disabled && input.getClientRects().length)input.focus();});
  return {
    read:()=>rows.map(row=>row.value),write,enable,clear:()=>write([]),
    capture:()=>({members:rows.map(row=>row.value),focus:rows.findIndex(row=>row.input===document.activeElement)}),
    restore:snapshot=>{write(snapshot.members);if(snapshot.focus>=0 && document.activeElement===document.body){const input=rows[snapshot.focus]?.input;if(input && !input.disabled && input.getClientRects().length)input.focus({preventScroll:true});}},
  };
}
"#;
