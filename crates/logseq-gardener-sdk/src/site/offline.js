// The Save pages for offline reading and Remove offline copies items in the More menu, and the
// saved-copy notice. Nothing is registered or stored until the reader chooses Save.
(() => {
const $ = selector => document.querySelector(selector), save = $('[data-lsg-offline-save]'), remove = $('[data-lsg-offline-remove]'), sw = navigator.serviceWorker;
const worker = new URL('lsg-sw.js', document.baseURI), scope = new URL('./', worker).href, name = 'lsg-offline:' + new URL(scope).pathname;
// Service workers exist only on HTTPS and localhost; a worker at an origin root would control
// every other site on that origin.
if (!save || !remove || !sw || !/^https?:$/.test(location.protocol) || name.endsWith(':/')) return;
const show = on => { save.hidden = on; remove.hidden = !on; };
const mine = async () => { const found = await sw.getRegistration(scope); return found?.scope === scope ? found : null; };
const say = (text, wait = 50) => {
let line = $('[data-lsg-offline-status]');
if (!line) {
line = document.createElement('p');
line.setAttribute('role', 'status');
line.setAttribute('data-lsg-offline-status', '');
$('main')?.prepend(line);
}
setTimeout(() => { line.textContent = text; }, wait);
};
const done = () => {
const menu = $('#lsg-more-menu'), more = $('[data-lsg-more]');
if (menu) menu.hidden = true;
more?.setAttribute('aria-expanded', 'false');
more?.focus();
};
mine().then(found => show(!!found), () => {});
save.addEventListener('click', async () => {
done();
try {
await sw.register(worker.href, { scope });
show(true);
say('Saving is on. Pages you open from now on are kept for offline reading.');
} catch {
say('This browser could not turn on offline reading.');
}
});
remove.addEventListener('click', async () => {
done();
try { await (await mine())?.unregister(); } catch {}
await caches.delete(name);
show(false);
say('Offline copies removed.');
});
// A page answered from a stored copy says when the copy was saved.
const entry = performance.getEntriesByType('navigation')[0];
if (sw.controller && (!navigator.onLine || entry?.serverTiming?.some(timing => timing.name === 'lsg-offline'))) {
caches.match(location.href, { cacheName: name }).then(copy => {
const saved = +copy?.headers.get('x-lsg-saved');
if (saved) say('Saved copy from ' + new Date(saved).toLocaleDateString(undefined, { day: 'numeric', month: 'long' }), 0);
}, () => {});
}
})();
