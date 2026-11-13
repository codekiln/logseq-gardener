// Offline reading is turned off for this garden: delete the garden's copies and unregister.
addEventListener('install', () => skipWaiting());
addEventListener('activate', event => event.waitUntil((async () => {
await caches.delete('lsg-offline:' + new URL(registration.scope).pathname);
await registration.unregister();
})()));
