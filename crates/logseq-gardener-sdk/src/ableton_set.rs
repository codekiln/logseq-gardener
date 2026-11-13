//! An Ableton Live Set written as pages: the Set page, and one page per group and track.
//!
//! A Set page carries `logseq-entity:: [[Logseq/Entity/Ableton/Set]]` ([`SET_TYPE`]) and embeds
//! each top-level group and track page once, in Live's order, with `{{embed [[page]]}}`. A group
//! page ([`GROUP_TYPE`]) embeds its tracks the same way. Every one of these pages has one
//! heading that names it, and writes its settings as block properties of that heading:
//! [`SET_SETTINGS`] on a Set, [`TRACK_SETTINGS`] and [`DEVICES`] on a track or group.
//!
//! ```
//! use logseq_gardener_sdk::ableton_set::{embedded_pages, heading};
//!
//! let track = lsdoc::parse_to_projection(
//!     "- # 1A-Audio\n  type:: audio\n  arm:: false\n  devices:: Compressor, Reverb\n",
//! );
//! let found = heading(&track.blocks).unwrap();
//! assert_eq!(found.title, "1A-Audio");
//! assert_eq!(found.setting("type"), Some("audio"));
//! assert_eq!(found.devices(), ["Compressor", "Reverb"]);
//!
//! let set = lsdoc::parse_to_projection("- # Set\n\t- {{embed [[Set/Track/1A-Audio]]}}\n");
//! assert_eq!(embedded_pages(&set.blocks), ["Set/Track/1A-Audio"]);
//! ```
use crate::ast::{Block, Inline};
use crate::outline::{Outline, bullet_properties};

/// The entity type that marks a Set page.
pub const SET_TYPE: &str = "Logseq/Entity/Ableton/Set";
/// The entity type that marks a group track page, which embeds its tracks.
pub const GROUP_TYPE: &str = "Logseq/Entity/Ableton/Track/Group";
/// The entity type that marks an audio, MIDI or return track page.
pub const TRACK_TYPE: &str = "Logseq/Entity/Ableton/Track/Track";

/// The block properties of a Set's heading that describe the Set.
pub const SET_SETTINGS: [&str; 3] = ["live-version", "tempo", "time-signature"];
/// The block properties of a track's or group's heading that describe it, besides
/// [`DEVICES`].
pub const TRACK_SETTINGS: [&str; 8] = [
    "type",
    "audio-from",
    "audio-to",
    "midi-from",
    "midi-to",
    "midi-channel",
    "monitor",
    "arm",
];
/// The [`TRACK_SETTINGS`] that say where a track's signal comes from and goes to. A value may
/// start with a reference to another track's page ([`leading_reference`]).
pub const ROUTING_SETTINGS: [&str; 4] = ["audio-from", "audio-to", "midi-from", "midi-to"];
/// The block property that lists a track's devices in chain order, separated by commas.
pub const DEVICES: &str = "devices";

/// The heading that names a Set, group or track page, with the block properties under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// The heading's text.
    pub title: String,
    /// The heading's block properties in source order, with keys as Logseq stores them: lower
    /// case, `_` read as `-`.
    pub properties: Vec<(String, String)>,
}

impl Heading {
    /// The value of the block property `key`, when the heading has one with text. The last
    /// value of a repeated key wins, as in Logseq.
    #[must_use]
    pub fn setting(&self, key: &str) -> Option<&str> {
        self.properties
            .iter()
            .rev()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
            .filter(|value| !value.is_empty())
    }

    /// The devices of `devices::` in chain order.
    #[must_use]
    pub fn devices(&self) -> Vec<&str> {
        self.setting(DEVICES)
            .into_iter()
            .flat_map(|value| value.split(','))
            .map(str::trim)
            .filter(|device| !device.is_empty())
            .collect()
    }
}

/// The first heading block of a page's outline. `None` when the page has no heading block.
#[must_use]
pub fn heading(blocks: &[Block]) -> Option<Heading> {
    let outline = Outline::from_blocks(blocks);
    let first = outline
        .blocks
        .iter()
        .find(|block| block.heading.is_some())?;
    let Block::Bullet { inline, .. } = &blocks[first.index] else {
        return None;
    };
    let mut title = String::new();
    crate::podcast::plain_text(inline, &mut title, 0);
    Some(Heading {
        title: title.split_whitespace().collect::<Vec<_>>().join(" "),
        properties: bullet_properties(blocks, first.index)
            .map(|(key, value)| (key.to_lowercase().replace('_', "-"), value.to_owned()))
            .collect(),
    })
}

/// The page a routing value starts with and the text after it: `[[Set/Track/1n2]] Track In`
/// gives `("Set/Track/1n2", "Track In")`. `None` when the value does not start with a page
/// reference.
///
/// ```
/// use logseq_gardener_sdk::ableton_set::leading_reference;
///
/// assert_eq!(leading_reference("[[Set/Track/1n2]] Track In"), Some(("Set/Track/1n2", "Track In")));
/// assert_eq!(leading_reference("Ext. In 3"), None);
/// ```
#[must_use]
pub fn leading_reference(value: &str) -> Option<(&str, &str)> {
    let (name, rest) = value.trim().strip_prefix("[[")?.split_once("]]")?;
    let name = name.trim();
    (!name.is_empty() && !name.contains("[[")).then_some((name, rest.trim()))
}

/// The names of the pages a page embeds with `{{embed [[page]]}}`, in page order. Block
/// embeds, `{{embed ((id))}}`, are left out.
#[must_use]
pub fn embedded_pages(blocks: &[Block]) -> Vec<String> {
    blocks
        .iter()
        .filter_map(|block| match block {
            Block::Bullet { inline, .. } => Some(inline),
            _ => None,
        })
        .flatten()
        .filter_map(|node| match node {
            Inline::Macro { name, args, .. } if name.eq_ignore_ascii_case("embed") => {
                let argument = args.join(", ");
                let name = argument
                    .trim()
                    .strip_prefix("[[")?
                    .strip_suffix("]]")?
                    .trim();
                (!name.is_empty()).then(|| name.to_owned())
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocks(source: &str) -> Vec<Block> {
        lsdoc::parse_to_projection(source).blocks
    }

    #[test]
    fn the_first_heading_names_the_page_and_holds_its_settings() {
        let found = heading(&blocks(
            "logseq-entity:: [[Logseq/Entity/Ableton/Track/Track]]\n\n- # LPP to **MF**\n  type:: midi\n  MIDI_Channel:: 1\n  arm:: true\n  arm:: false\n  monitor::\n\t- Carries notes.\n\t- ## Later heading\n\t  type:: audio\n",
        ))
        .unwrap();
        assert_eq!(found.title, "LPP to MF");
        assert_eq!(found.setting("type"), Some("midi"));
        // Keys are read as Logseq stores them, and the last value of a repeated key wins.
        assert_eq!(found.setting("midi-channel"), Some("1"));
        assert_eq!(found.setting("arm"), Some("false"));
        // A key without a value, and a key the heading does not have, give nothing.
        assert_eq!(found.setting("monitor"), None);
        assert_eq!(found.setting("audio-from"), None);
        assert!(found.devices().is_empty());
    }

    #[test]
    fn a_page_without_a_heading_has_none() {
        assert_eq!(heading(&blocks("- plain\n  type:: audio\n")), None);
        assert_eq!(heading(&blocks("")), None);
    }

    #[test]
    fn devices_keep_chain_order() {
        let found = heading(&blocks(
            "- # commentary\n  devices:: Multband & Limiter, Voice Enhancer, ,\n",
        ))
        .unwrap();
        assert_eq!(found.devices(), ["Multband & Limiter", "Voice Enhancer"]);
    }

    #[test]
    fn a_routing_value_may_start_with_a_page_reference() {
        assert_eq!(
            leading_reference("  [[ Set/Track/1n2 ]]  Post Mixer "),
            Some(("Set/Track/1n2", "Post Mixer"))
        );
        assert_eq!(
            leading_reference("[[Set/Track/1n2]]"),
            Some(("Set/Track/1n2", ""))
        );
        for value in [
            "Ext. In 3",
            "to [[Set/Track/1n2]]",
            "[[]] Track In",
            "[[Set/Track/1n2",
            "[[a [[b]] c]]",
            "",
        ] {
            assert_eq!(leading_reference(value), None, "{value}");
        }
    }

    #[test]
    fn page_embeds_are_listed_in_page_order() {
        let found = embedded_pages(&blocks(
            "- # Set\n\t- Text about [[Set/Track/Linked]]\n\t- {{embed [[Set/Track/B]]}}\n\t- {{EMBED [[ Set/Track/A ]]}}\n\t\t- {{embed [[Set/Track/Nested]]}}\n\t- {{embed ((65f0c1d2-0000-4000-8000-000000000001))}}\n\t- {{embed [[]]}}\n\t- {{query [[Set/Track/Query]]}}\n",
        ));
        assert_eq!(found, ["Set/Track/B", "Set/Track/A", "Set/Track/Nested"]);
    }
}
