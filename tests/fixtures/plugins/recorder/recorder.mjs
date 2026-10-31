// A test plugin for lsg's plugin host. It keeps one <audio> element in a persistent region and
// plays the tone a page names in its recording:: property, so tests can check that playback
// continues while the reader moves between pages. It calls every host function.
const SECONDS = 30;

// A sine tone as a WAV blob, so the fixture needs no audio file.
function tone(name) {
  const rate = 8000;
  const frequency = 220 + [...name].reduce((sum, c) => sum + c.charCodeAt(0), 0) % 440;
  const samples = rate * SECONDS;
  const view = new DataView(new ArrayBuffer(44 + samples));
  const text = (offset, value) => [...value].forEach((c, i) => view.setUint8(offset + i, c.charCodeAt(0)));
  text(0, 'RIFF');
  view.setUint32(4, 36 + samples, true);
  text(8, 'WAVEfmt ');
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, rate, true);
  view.setUint32(28, rate, true);
  view.setUint16(32, 1, true);
  view.setUint16(34, 8, true);
  text(36, 'data');
  view.setUint32(40, samples, true);
  for (let i = 0; i < samples; i++) view.setUint8(44 + i, 128 + Math.round(40 * Math.sin(2 * Math.PI * frequency * i / rate)));
  return URL.createObjectURL(new Blob([view], { type: 'audio/wav' }));
}

export default {
  id: 'fixture-recorder',
  apiVersion: 1,
  setup(host) {
    if (host.apiVersion !== 1) throw new Error('unexpected host API version');
    const region = host.persistentRegion('player');
    const box = region.element;
    box.dataset.base = host.baseURL;
    const audio = host.document.createElement('audio');
    audio.controls = true;
    audio.loop = true;
    box.append(audio);
    let adoptions = 0;
    const stop = host.onPageAdopted(({ page, main, revision, metadata, signal }) => {
      adoptions += 1;
      box.dataset.adoptions = String(adoptions);
      box.dataset.page = page.url;
      box.dataset.revision = revision;
      main.dataset.fixtureRecorder = page.title;
      const name = metadata.recording;
      if (name && audio.dataset.recording !== name) {
        audio.dataset.recording = name;
        audio.src = tone(name);
        audio.play().catch(() => {});
      }
      // A page-scoped listener: the host aborts the signal before the page is replaced.
      host.document.addEventListener('lsg-fixture-ping', () => {
        box.dataset.pings = String(Number(box.dataset.pings || 0) + 1);
      }, { signal });
    });
    return () => {
      stop();
      audio.pause();
      region.dispose();
    };
  },
};
