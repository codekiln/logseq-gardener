//! The first-party `ableton-set` plugin: which pages get an entry and what it holds.
use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{SiteOptions, publish_site_with_options};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const SET: &str = "logseq-entity:: [[Logseq/Entity/Ableton/Set]]\n\n- # Evening Set\n  live-version:: 11.3.43\n  tempo:: 96\n  time-signature:: 4/4\n  source:: evening.als\n\t- How the performance was wired. [[Live/Set/Track/Linked Only]]\n\t- {{embed [[Live/Set/Track/Synths]]}}\n\t- {{embed [[live/set/track/commentary]]}}\n\t- {{embed [[Live/Set/Track/Private]]}}\n\t- {{embed [[Live/Set/Track/Missing]]}}\n\t- {{embed [[Live/Notes]]}}\n\t- {{embed [[Live/Set]]}}\n\t- {{embed [[Live/Set/Track/Return]]}}\n";

struct Fixture(PathBuf);

impl Fixture {
    /// A garden with one Set: the group `Synths` holding `MicroFreak MIDI`, the ungrouped
    /// track `Commentary`, a return track, and embeds that name no track. `extra` goes into its
    /// `config.edn` map.
    fn new(extra: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-ableton-set-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let fixture = Self(root);
        fixture.write(
            "logseq/config.edn",
            &format!("{{:publishing/all-pages-public? true {extra}}}"),
        );
        for (name, text) in [
            ("Live___Set.md", SET),
            (
                "Live___Set___Track___Synths.md",
                "logseq-entity:: [[Logseq/Entity/Ableton/Track/Group]]\n\n- # Synths\n  type:: group\n  audio-to:: [[live/set/track/commentary]]\n\t- {{embed [[Live/Set/Track/Synths/MicroFreak MIDI]]}}\n\t- {{embed [[Live/Set/Track/Synths]]}}\n\t- {{embed [[Live/Set]]}}\n",
            ),
            (
                "Live___Set___Track___Synths___MicroFreak MIDI.md",
                "logseq-entity:: [[Logseq/Entity/Ableton/Track/Track]]\n\n- # MicroFreak & <MIDI>\n  type:: midi\n  midi-from:: [[Live/Set/Track/Private]] Post Mixer\n  midi-to:: MidiOut/External.Dev:Arturia MicroFreak/0\n  midi-channel:: 1\n  monitor:: auto\n  arm:: true\n\t- Carries the sequencer's notes. TRACK_BODY\n",
            ),
            (
                "Live___Set___Track___Commentary.md",
                "logseq-entity:: [[Logseq/Entity/Ableton/Track/Track]]\n\n- # Commentary\n  type:: audio\n  audio-from:: AudioIn/External/S0\n  audio-to:: [[Live/Set/Track/Synths]] Track In\n  monitor:: off\n  arm:: false\n  devices:: Compressor, EQ Eight, Limiter\n\t- Records the afterword.\n",
            ),
            (
                // A track with no heading is named by its page, and states nothing else.
                "Live___Set___Track___Return.md",
                "logseq-entity:: [[Logseq/Entity/Ableton/Track/Track]]\n\n- A return track without a heading.\n  type:: return\n",
            ),
            (
                "Live___Set___Track___Private.md",
                "public:: false\nlogseq-entity:: [[Logseq/Entity/Ableton/Track/Track]]\n\n- # PRIVATE_TRACK\n  type:: audio\n",
            ),
            (
                "Live___Set___Track___Linked Only.md",
                "logseq-entity:: [[Logseq/Entity/Ableton/Track/Track]]\n\n- # Linked Only\n  type:: audio\n",
            ),
            ("Live___Notes.md", "- # Notes\n  type:: audio\n"),
            (
                "Live___Empty Set.md",
                "logseq-entity:: [[Logseq/Entity/Ableton/Set]]\n\n- # Empty\n\t- {{embed [[Live/Notes]]}}\n",
            ),
        ] {
            fixture.write(&format!("pages/{name}"), text);
        }
        fixture
    }

    fn write(&self, name: &str, text: &str) {
        let path = self.0.join("garden").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn publish(&self, site: &str) -> (PathBuf, Vec<String>) {
        let output = self.0.join(site);
        let report = publish_site_with_options(
            self.0.join("garden"),
            &output,
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Live"], &[]).unwrap(),
            &SiteOptions::default(),
        )
        .unwrap();
        let diagnostics = report
            .diagnostics
            .iter()
            .map(|found| format!("{}: {}", found.source.display(), found.message))
            .collect();
        (output, diagnostics)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn manifest(site: &Path) -> Value {
    serde_json::from_slice(&fs::read(site.join("lsg-site.json")).unwrap()).unwrap()
}

fn url(site: &Path, title: &str) -> String {
    manifest(site)["pages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|page| page["title"] == title)
        .unwrap_or_else(|| panic!("{title} is published"))["url"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// The page's `ableton-set` entry, or `Value::Null` without one.
fn entry(site: &Path, title: &str) -> Value {
    let html = fs::read_to_string(site.join(url(site, title))).unwrap();
    let start = html
        .find("<script type=\"application/json\" data-lsg-page>")
        .expect("pages carry metadata");
    let json = &html[start..];
    let json = &json[json.find('>').unwrap() + 1..json.find("</script>").unwrap()];
    let metadata: Value = serde_json::from_str(json).unwrap();
    metadata["plugins"]["ableton-set"].clone()
}

#[test]
fn a_set_entry_holds_its_groups_and_tracks_in_page_order() {
    let fixture = Fixture::new(":lsg/plugins [:ableton-set]");
    let (site, diagnostics) = fixture.publish("site");
    let embedded = Path::new(env!("CARGO_MANIFEST_DIR")).join("plugins/ableton-set");
    for file in ["ableton-set.mjs", "ableton-set.css"] {
        assert_eq!(
            fs::read(site.join("plugins/ableton-set").join(file)).unwrap(),
            fs::read(embedded.join(file)).unwrap(),
            "{file}"
        );
    }
    assert_eq!(
        manifest(&site)["plugins"],
        json!([{
            "id": "ableton-set",
            "version": logseq_gardener_sdk::VERSION,
            "module": "plugins/ableton-set/ableton-set.mjs",
            "styles": ["plugins/ableton-set/ableton-set.css"],
            "activation": "on-demand",
        }])
    );
    // A group with the track it embeds, a track outside any group, and a track without a
    // heading. A withheld page, a missing page, a page that is not a track, the Set itself and
    // a page the Set only links to add no column; a group that embeds itself or its Set adds
    // none inside.
    assert_eq!(
        entry(&site, "Live/Set"),
        json!({
            "title": "Evening Set",
            "live-version": "11.3.43",
            "tempo": "96",
            "time-signature": "4/4",
            "tracks": [
                {
                    "title": "Synths",
                    "url": url(&site, "Live/Set/Track/Synths"),
                    "type": "group",
                    // A routing that names a published page, with nothing after it.
                    "audio-to": {"title": "Commentary", "url": url(&site, "Live/Set/Track/Commentary")},
                    "tracks": [{
                        "title": "MicroFreak & <MIDI>",
                        "url": url(&site, "Live/Set/Track/Synths/MicroFreak MIDI"),
                        "type": "midi",
                        // A routing that names a withheld page: its name as text, no URL.
                        "midi-from": "Private Post Mixer",
                        "midi-to": "MidiOut/External.Dev:Arturia MicroFreak/0",
                        "midi-channel": "1",
                        "monitor": "auto",
                        "arm": true,
                    }],
                },
                {
                    "title": "Commentary",
                    "url": url(&site, "Live/Set/Track/Commentary"),
                    "type": "audio",
                    "audio-from": "AudioIn/External/S0",
                    "audio-to": {
                        "title": "Synths",
                        "url": url(&site, "Live/Set/Track/Synths"),
                        "detail": "Track In",
                    },
                    "monitor": "off",
                    "arm": false,
                    "devices": ["Compressor", "EQ Eight", "Limiter"],
                },
                {"title": "Return", "url": url(&site, "Live/Set/Track/Return")},
            ],
        })
    );
    // Only a Set page gets an entry, so the browser imports no plugin code on a track page.
    for title in [
        "Live/Set/Track/Synths",
        "Live/Set/Track/Commentary",
        "Live/Notes",
        "Live",
    ] {
        assert_eq!(entry(&site, title), Value::Null, "{title}");
    }
    // A Set that embeds no track has no entry, and the build says so once.
    assert_eq!(entry(&site, "Live/Empty Set"), Value::Null);
    let about_sets: Vec<&String> = diagnostics
        .iter()
        .filter(|message| message.contains("ableton-set:"))
        .collect();
    assert_eq!(about_sets.len(), 1, "{diagnostics:?}");
    assert!(
        about_sets[0].starts_with("pages/Live___Empty Set.md: ")
            && about_sets[0].contains("embeds no published track or group page"),
        "{about_sets:?}"
    );
    // The withheld track reaches no generated file.
    for path in walk(&site) {
        let text = String::from_utf8_lossy(&fs::read(&path).unwrap()).into_owned();
        assert!(!text.contains("PRIVATE_TRACK"), "{}", path.display());
    }
}

#[test]
fn a_hidden_setting_stays_out_of_the_entry() {
    let fixture = Fixture::new(
        ":lsg/plugins [:ableton-set] :block-hidden-properties #{:midi-to :tempo :devices}",
    );
    let (site, _) = fixture.publish("site");
    let set = entry(&site, "Live/Set");
    assert_eq!(set.get("tempo"), None);
    assert_eq!(set["live-version"], "11.3.43");
    let midi = &set["tracks"][0]["tracks"][0];
    assert_eq!(midi.get("midi-to"), None);
    assert_eq!(midi["midi-from"], "Private Post Mixer");
    assert_eq!(set["tracks"][1].get("devices"), None);
}

#[test]
fn the_outline_is_the_same_with_and_without_the_plugin() {
    let with = Fixture::new(":lsg/plugins [:ableton-set]");
    let without = Fixture::new("");
    let (with_site, _) = with.publish("site");
    let (without_site, _) = without.publish("site");
    let main = |site: &Path| {
        let html = fs::read_to_string(site.join(url(site, "Live/Set"))).unwrap();
        let start = html.find("<main").unwrap();
        html[start..html.find("</main>").unwrap()].to_owned()
    };
    let outline = main(&with_site);
    assert_eq!(outline, main(&without_site));
    // The embedded track pages stay in the outline for readers without scripts and for search.
    assert!(outline.contains("TRACK_BODY"), "{outline}");
    // Without the key the page carries no plugin metadata and the site holds no plugin file.
    let plain = fs::read_to_string(without_site.join(url(&without_site, "Live/Set"))).unwrap();
    assert!(!plain.contains("data-lsg-page"));
    assert!(!without_site.join("plugins").exists());
}

fn walk(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(walk(&path));
        } else {
            found.push(path);
        }
    }
    found
}
