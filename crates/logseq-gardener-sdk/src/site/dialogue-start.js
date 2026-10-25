(() => {
  if (window.logseqGardenerDialogueInstalled || window.logseqGardenerDialogueLoading) return;
  window.logseqGardenerDialogueLoading = true;
  const source = document.querySelector('link[rel="lsg-dialogue"]').href;
  const load = () => {
    const script = document.createElement('script');
    script.src = source;
    document.head.append(script);
  };
  if (document.readyState === 'complete') load();
  else window.addEventListener('load', load, { once: true });
})();
