// Keeps a copy of each page a reader opens after choosing Save pages for offline reading.
// Network first: an online reader always gets the current page; a copy answers only when the
// network fails or is slow. lsg fills in the build identifier and the limits.
const BUILD = '__BUILD__', PAGES = __PAGES__, BYTES = __BYTES__, ITEM = __ITEM__, AGE = __AGE__, WAIT = __WAIT__;
const SCOPE = new URL(registration.scope), NAME = 'lsg-offline:' + SCOPE.pathname, RECORD = SCOPE.href + '?lsg-offline-record';
// The record maps each stored address to [size, saved time, last use]; `build` is the worker
// version that last checked the stored pages.
let record = null, queue = Promise.resolve(), dirty = false;
const serial = task => (queue = queue.then(task, task));
const open = () => caches.open(NAME);
const load = async () => {
if (record) return record;
const cache = await open();
try {
const stored = await (await cache.match(RECORD)).json();
if (!stored || typeof stored.pages !== 'object') throw 0;
record = stored;
} catch {
// A missing or unreadable record: start over rather than guess which copies exist.
await caches.delete(NAME);
record = { build: '', pages: {} };
}
const old = Date.now() - AGE;
for (const [url, [, saved]] of Object.entries(record.pages)) if (saved < old) await drop(url);
return record;
};
const save = async () => {
if (!dirty) return;
dirty = false;
await (await open()).put(RECORD, new Response(JSON.stringify(record)));
};
const drop = async url => {
delete record.pages[url];
dirty = true;
await (await open()).delete(url);
};
// Drop the least recently used copies until `keep` copies and `bytes` remain.
const trim = async (keep, bytes) => {
const list = Object.entries(record.pages).sort((a, b) => a[1][2] - b[1][2]);
let total = list.reduce((sum, [, [size]]) => sum + size, 0);
for (const [url, [size]] of list) {
if (list.length <= keep && total <= bytes) break;
await drop(url);
list.length--;
total -= size;
}
};
const store = (url, response) => serial(async () => {
await load();
const length = +response.headers.get('content-length');
if (length > ITEM) return;
const body = await response.blob();
if (body.size > ITEM) return;
const now = Date.now(), headers = new Headers(response.headers);
headers.set('x-lsg-saved', now);
const copy = () => new Response(body, { status: 200, statusText: response.statusText, headers });
await trim(PAGES - 1, BYTES - body.size);
try {
await (await open()).put(url, copy());
} catch {
// Out of space: drop the older half and try once more, then skip this copy.
await trim(Object.keys(record.pages).length >> 1, BYTES);
try { await (await open()).put(url, copy()); } catch { await save(); return; }
}
record.pages[url] = [body.size, now, now];
dirty = true;
await save();
});
const forget = url => serial(async () => { await load(); if (record.pages[url]) { await drop(url); await save(); } });
const answer = url => serial(async () => {
await load();
const copy = record.pages[url] && await (await open()).match(url);
if (!copy) return null;
record.pages[url][2] = Date.now();
dirty = true;
await save();
const headers = new Headers(copy.headers);
headers.set('server-timing', 'lsg-offline');
return new Response(copy.body, { status: 200, statusText: copy.statusText, headers });
});
// A new version asks the network about every stored page, two at a time and after the reader's
// own request, and deletes those it no longer serves. Any other answer keeps the copy.
const check = async () => {
const pages = await serial(async () => {
await load();
if (record.build === BUILD) return [];
record.build = BUILD;
dirty = true;
await save();
return Object.keys(record.pages);
});
const next = async () => {
for (let url; (url = pages.shift());) {
try {
const response = await fetch(url, { method: 'HEAD', cache: 'no-store' });
if (response.status === 404 || response.status === 410) await forget(url);
} catch {}
}
};
await Promise.all([next(), next()]);
};
const kept = (request, url) => request.method === 'GET' && url.origin === SCOPE.origin && url.pathname.startsWith(SCOPE.pathname)
&& url.href !== RECORD && !request.headers.has('range') && !/^(audio|video|track)$/.test(request.destination)
&& !/\/lsg-search-[^/]*\.json$/.test(url.pathname);
const media = response => /^(audio|video)\//.test(response.headers.get('content-type') || '');
addEventListener('install', () => skipWaiting());
addEventListener('activate', event => event.waitUntil((async () => {
await registration.navigationPreload?.enable();
await clients.claim();
})()));
addEventListener('fetch', event => {
const request = event.request, url = new URL(request.url);
if (!kept(request, url)) return;
const key = url.href.split('#')[0], navigate = request.mode === 'navigate';
const network = (async () => (navigate && await event.preloadResponse) || fetch(request))();
const saved = response => {
if (response.status === 200 && response.type === 'basic' && !media(response)) event.waitUntil(store(key, response.clone()));
else if (response.status === 404 || response.status === 410) event.waitUntil(forget(key));
return response;
};
event.respondWith((async () => {
const late = navigate && new Promise(resolve => setTimeout(resolve, WAIT, 'late'));
try {
const first = await Promise.race([network, late].filter(Boolean));
if (first !== 'late') { event.waitUntil(check()); return saved(first); }
const copy = await answer(key);
if (copy) { event.waitUntil(network.then(saved, () => {})); return copy; }
return saved(await network);
} catch {
return (await answer(key)) || Response.error();
}
})());
});
