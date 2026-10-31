// Shows "n of m" from the build-time metadata on each listed page.
export default {
  id: 'fixture-episode-list',
  apiVersion: 1,
  setup(host) {
    host.onPageAdopted(({ main, metadata, signal }) => {
      if (!metadata.position) return;
      const note = host.document.createElement('p');
      note.dataset.lsgPlugin = 'fixture-episode-list';
      note.textContent = `${metadata.position} of ${metadata.of}`;
      main.append(note);
      signal.addEventListener('abort', () => note.remove());
    });
  },
};
