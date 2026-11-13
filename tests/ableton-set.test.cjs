// The ableton-set first-party plugin's entry reading. scripts/plugins/browser.py checks the
// drawn view in Chromium.
const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const loaded = import(pathToFileURL(path.join(__dirname, '../crates/logseq-gardener-sdk/plugins/ableton-set/ableton-set.mjs')));

const base = 'https://garden.example.org/site/';
// A Set with a group, a track inside the group and a track outside any group.
const entry = {
  title: 'Evening Set',
  'live-version': '11.3.43',
  tempo: '96',
  'time-signature': '4/4',
  tracks: [
    {
      title: 'Synths', url: 'Live/Set/Track/Synths/', type: 'group', 'audio-to': 'AudioOut/Master',
      tracks: [{
        title: 'MicroFreak MIDI', url: 'Live/Set/Track/Synths/MicroFreak%20MIDI/', type: 'midi',
        'midi-from': 'MidiIn/External.Dev:Launchpad Pro MK3/0', 'midi-to': 'MidiOut/External.Dev:Arturia MicroFreak/0',
        'midi-channel': '1', monitor: 'auto', arm: true,
      }],
    },
    {
      title: 'Commentary', url: 'Live/Set/Track/Commentary/', type: 'audio', 'audio-from': 'AudioIn/External/S0',
      'audio-to': { title: 'Synths', url: 'Live/Set/Track/Synths/', detail: 'Track In' }, monitor: 'off', arm: false, devices: ['Compressor', 'EQ Eight', 'Limiter'],
    },
  ],
};

test('setView reads a group, a grouped track and an ungrouped track in order', async () => {
  const { setView, trackCount } = await loaded;
  const view = setView(entry, base);
  assert.equal(view.title, 'Evening Set');
  assert.deepEqual(view.facts, ['Live 11.3.43', '96 BPM', '4/4']);
  assert.deepEqual(view.tracks.map((track) => track.title), ['Synths', 'Commentary']);
  assert.equal(trackCount(view.tracks), 3);
  const [group, commentary] = view.tracks;
  assert.equal(group.type, 'Group');
  assert.equal(group.href, 'https://garden.example.org/site/Live/Set/Track/Synths/');
  assert.deepEqual(group.rows, [{ key: 'audio-to', label: 'Audio to', value: 'AudioOut/Master', href: null }]);
  assert.equal(group.arm, null);
  const [midi] = group.tracks;
  assert.equal(midi.type, 'MIDI');
  assert.equal(midi.kind, 'midi');
  assert.equal(midi.href, 'https://garden.example.org/site/Live/Set/Track/Synths/MicroFreak%20MIDI/');
  // Rows follow Live's In/Out section: input, channel, monitor, output.
  assert.deepEqual(midi.rows.map((row) => [row.label, row.value]), [
    ['MIDI from', 'MidiIn/External.Dev:Launchpad Pro MK3/0'],
    ['MIDI channel', '1'],
    ['Monitor', 'auto'],
    ['MIDI to', 'MidiOut/External.Dev:Arturia MicroFreak/0'],
  ]);
  assert.equal(midi.arm, true);
  assert.equal(midi.tracks, null);
  assert.equal(commentary.type, 'Audio');
  // A routing to another track of the Set: its title, its page and Live's second box.
  assert.deepEqual(commentary.rows.at(-1), {
    key: 'audio-to', label: 'Audio to', value: 'Synths',
    href: 'https://garden.example.org/site/Live/Set/Track/Synths/', detail: 'Track In',
  });
  assert.equal(commentary.arm, false);
  assert.deepEqual(commentary.devices, ['Compressor', 'EQ Eight', 'Limiter']);
  assert.equal(commentary.tracks, null);
});

test('setView keeps what a track states and nothing else', async () => {
  const { setView } = await loaded;
  const view = setView({
    tracks: [
      { title: 'Bare', url: 'bare.html' },
      { title: 'Empty group', url: 'group.html', tracks: [] },
      { title: 'Odd', url: 'odd.html', type: 'Sidechain', monitor: 'sometimes', arm: 'true', devices: 'Reverb' },
      { title: 'Away', url: 'https://elsewhere.example.com/p.html', type: 'AUDIO' },
      { title: 'Script', url: 'javascript:alert(1)' },
      { title: '', url: 'untitled.html' },
      null,
      'text',
    ],
  }, base);
  assert.equal(view.title, '');
  assert.deepEqual(view.facts, []);
  assert.deepEqual(view.tracks.map((track) => [track.title, track.type, track.kind, track.href]), [
    ['Bare', '', '', 'https://garden.example.org/site/bare.html'],
    // A group without a written type is still a group.
    ['Empty group', 'Group', 'group', 'https://garden.example.org/site/group.html'],
    // A type the plugin does not know stays as written and gets no colour.
    ['Odd', 'Sidechain', '', 'https://garden.example.org/site/odd.html'],
    // A link off the site, or one that is not a page address, gives a name without a link.
    ['Away', 'Audio', 'audio', null],
    ['Script', '', '', null],
  ]);
  const odd = view.tracks[2];
  assert.deepEqual(odd.rows, [{ key: 'monitor', label: 'Monitor', value: 'sometimes', href: null }]);
  // `arm` is a boolean or nothing, and `devices` a list or nothing.
  assert.equal(odd.arm, null);
  assert.deepEqual(odd.devices, []);
  assert.deepEqual(view.tracks[1].tracks, []);
});

test('setView reads a routing that names another track', async () => {
  const { setView } = await loaded;
  const rows = (routing) => setView({ tracks: [{ title: 'T', url: 't.html', 'audio-to': routing }] }, base).tracks[0].rows;
  assert.deepEqual(rows({ title: '1n2', url: 'one.html' }),
    [{ key: 'audio-to', label: 'Audio to', value: '1n2', href: 'https://garden.example.org/site/one.html' }]);
  // A target off the site, or without an address, keeps its name and has no link.
  assert.deepEqual(rows({ title: '1n2', url: 'https://elsewhere.example.com/p.html', detail: 'Track In' }),
    [{ key: 'audio-to', label: 'Audio to', value: '1n2', href: null, detail: 'Track In' }]);
  assert.deepEqual(rows({ title: '1n2' }), [{ key: 'audio-to', label: 'Audio to', value: '1n2', href: null }]);
  // A target without a title, and a value of another kind, give no row.
  for (const routing of [{ url: 'one.html', detail: 'Track In' }, { title: '' }, {}, null, 7, ['1n2']]) {
    assert.deepEqual(rows(routing), [], JSON.stringify(routing));
  }
});

test('setView gives no view for a page without tracks', async () => {
  const { setView } = await loaded;
  for (const metadata of [{}, undefined, null, { tracks: [] }, { tracks: 'Synths' }, { title: 'Set' },
    { tracks: [{ url: 'p.html' }] }, { tracks: [{ title: 7 }] }]) {
    assert.equal(setView(metadata, base), null, JSON.stringify(metadata));
  }
});

test('setView stops at the depth the build reads', async () => {
  const { setView, trackCount } = await loaded;
  let tracks = [{ title: 'leaf', url: 'leaf.html' }];
  for (let depth = 0; depth < 20; depth += 1) tracks = [{ title: `group ${depth}`, url: 'g.html', tracks }];
  assert.equal(trackCount(setView({ tracks }, base).tracks), 9);
});
