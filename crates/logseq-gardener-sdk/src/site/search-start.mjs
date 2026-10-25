let o=window.logseqGardenerInitSearch?.options||{},p=false,h=false,w,c,a=0,q='';
const d=document,s='[data-lsg-search]',B=()=>d.querySelectorAll(s),id=()=>{const l=d.head.querySelector('[rel=lsg-search]');return l?.href+':'+l?.dataset.revision;};
function i(r,u={}){o={...o,...u};const v=id();if(v!==c){h=false;q='';}c=v;for(const b of B())b.hidden=!d.head.querySelector('[rel=lsg-search]')||location.protocol==='file:'||!window.HTMLDialogElement?.prototype.showModal;}
window.logseqGardenerInitSearch=i;i();
const t=d.createElement('span');t.id='lsg-search-load';t.role='status';
async function f(v){
 const k=v.type==='keydown',x=v.target.closest?.(s);
 if(k&&p&&h){v.stopPropagation();if(v.isComposing||v.keyCode===229)return;if(v.key==='Escape'){v.preventDefault();h=false;q='';t.textContent='';return;}if(!v.ctrlKey&&!v.metaKey&&!v.altKey&&!v.isComposing){if(v.key.length===1&&q.length<512)q+=v.key;else if(v.key==='Backspace')q=[...q].slice(0,-1).join('');v.preventDefault();return;}}
 if(k&&(!(v.ctrlKey||v.metaKey)||v.key.toLowerCase()!=='k'||v.altKey||v.shiftKey||v.isComposing))return;
 if(!k&&!x)return;
 if(![...B()].some(b=>!b.hidden))return;
 v.preventDefault();h=true;w=id();if(p)return;p=true;
 t.textContent='Loading search…';d.body.append(t);
 try{
  const m=await import(new URL('__LSG_SEARCH_UI__',import.meta.url)+(a++?'?retry='+a:''));
  delete window.logseqGardenerInitSearch;m.installSearch(window,d);
  d.removeEventListener('click',f);d.removeEventListener('keydown',f,true);
  window.logseqGardenerInitSearch(d,o);
  if(h&&w===id()){d.dispatchEvent(new KeyboardEvent('keydown',{key:'k',ctrlKey:true,bubbles:true}));const z=d.getElementById('lsg-search-input');if(z){z.value=q;z.dispatchEvent(new Event('input',{bubbles:true}));}}
  t.remove();
 }catch{window.logseqGardenerInitSearch=i;t.textContent=h&&w===id()?'Search could not load. Retry Search or reload.':'';}
 p=false;
}
d.addEventListener('click',f);d.addEventListener('keydown',f,true);
