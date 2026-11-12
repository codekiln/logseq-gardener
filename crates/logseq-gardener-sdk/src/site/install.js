// Shows the Install app item in the More menu when the browser offers installation.
// Chromium browsers fire beforeinstallprompt; Safari and Firefox never do, so the item stays hidden.
(() => {
  const item = document.querySelector('[data-lsg-install]');
  if (!item) return;
  const installed = () => matchMedia('(display-mode: standalone)').matches || navigator.standalone === true;
  let offer = null;
  window.addEventListener('beforeinstallprompt', event => {
    event.preventDefault();
    if (installed()) return;
    offer = event;
    item.hidden = false;
  });
  item.addEventListener('click', () => {
    const event = offer;
    offer = null;
    item.hidden = true;
    const menu = document.getElementById('lsg-more-menu'), more = document.querySelector('[data-lsg-more]');
    if (menu) menu.hidden = true;
    if (more) { more.setAttribute('aria-expanded', 'false'); more.focus(); }
    if (event) event.prompt().catch(() => {});
  });
})();
