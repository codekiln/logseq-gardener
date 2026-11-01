// Inline in every page's head: applies the saved shell layout and theme before the first paint,
// so a page never paints in the wrong layout or appearance. lsg-theme.js, loaded with defer,
// wires the theme menu afterwards. The build drops these comment lines; keep the code minified.
// The phone overlay always starts closed; the right sidebar's column takes its width here, and its
// cards load after load.
(()=>{const r=document.documentElement,a=(k,v)=>r.setAttribute('data-lsg-'+k,v),g=k=>{try{return localStorage.getItem('logseq-gardener-'+k)}catch{}},m=q=>{try{return window.matchMedia?.(q)?.matches}catch{}},w=m('(min-width: 640px)');if(w&&g('left-sidebar')==='open')a('left-sidebar','open');if(g('wide')==='true')a('wide','true');try{const b=JSON.parse(g('right-sidebar')||'null');if(b&&w){a('right-sidebar',b.open?'open':'closed');if(b.width>0)r.style.setProperty('--lsg-right-width',b.width+'px')}}catch{}let c=g('theme');if(c!=='light'&&c!=='dark')c='system';a('theme',c==='system'?m('(prefers-color-scheme: dark)')?'dark':'light':c);a('theme-mode',c)})();
