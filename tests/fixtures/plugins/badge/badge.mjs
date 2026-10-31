// A test plugin for lsg's plugin host, loaded on demand: the host imports it on the first page
// whose badge:: property it may read, and it labels each such page.
export default {
  id: 'fixture-badge',
  apiVersion: 1,
  setup(host) {
    return host.onPageAdopted(({ main, metadata }) => {
      if (!metadata.badge) return;
      const badge = host.document.createElement('span');
      badge.className = 'fixture-badge';
      badge.textContent = metadata.badge;
      main.querySelector('h1')?.after(badge);
    });
  },
};
