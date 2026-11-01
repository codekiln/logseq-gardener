// Inline at the end of every page's body: opens the collapsed blocks around the element the
// address's fragment names before the page paints. Native fragment navigation does not run when a
// host swaps page content, so hosts can call window.logseqGardenerRevealFragment(fragment,
// {scroll}) before their own focus and scroll handling. The build drops these comment lines; keep
// the code minified.
(()=>{function r(f=location.hash,{scroll:s=true}={}){let i;try{i=decodeURIComponent(f.replace(/^#/,''))}catch{return null}const t=i&&document.getElementById(i);if(!t)return null;for(let p=t.parentElement;p;p=p.parentElement){if(p.tagName==='DETAILS')p.open=true;p.classList?.remove('folded')}if(s)t.scrollIntoView();return t}window.logseqGardenerRevealFragment=r;window.addEventListener('hashchange',()=>r());r()})();
