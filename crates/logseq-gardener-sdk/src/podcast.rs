//! A podcast episode page: the recording it plays and the title it shows.
//!
//! A page is an episode when its `logseq-entity::` value names [`EPISODE_TYPE`] (see
//! [`crate::entity_types`]). Its recording is the first audio embed, `![label](url.mp3)`,
//! among the descendants of the page's first heading block, and its title is that heading's
//! text.
//!
//! ```
//! use logseq_gardener_sdk::podcast::recording;
//!
//! let parsed = lsdoc::parse_to_projection(
//!     "- # The Unstable over USB\n\t- Notes\n\t- ![Episode recording](https://example.com/full.mp3)\n",
//! );
//! let found = recording(&parsed.blocks).unwrap();
//! assert_eq!(found.destination, "https://example.com/full.mp3");
//! assert_eq!(found.title, "The Unstable over USB");
//! ```
use crate::ast::{Block, Inline, ListItem, Url};
use crate::outline::Outline;

/// The entity type that marks a podcast episode page.
pub const EPISODE_TYPE: &str = "Logseq/Entity/Podcast/Episode";

const MAX_DEPTH: usize = 64;

/// An episode's recording, as the page writes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recording {
    /// The audio embed's destination as written: an `https://` URL or a path relative to the
    /// page's file, such as `../assets/full.mp3`.
    pub destination: String,
    /// The text of the page's first heading.
    pub title: String,
}

/// The recording of a page's outline: the first audio embed among the descendants of its first
/// heading block, with that heading's text. `None` when the page has no heading block or no
/// audio embed under it.
pub fn recording(blocks: &[Block]) -> Option<Recording> {
    let outline = Outline::from_blocks(blocks);
    let first = outline
        .blocks
        .iter()
        .position(|block| block.heading.is_some())?;
    let heading = &outline.blocks[first];
    let Block::Bullet { inline, .. } = &blocks[heading.index] else {
        return None;
    };
    let mut title = String::new();
    plain_text(inline, &mut title, 0);
    let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
    let descendants = outline.blocks[first + 1..]
        .iter()
        .take_while(|block| block.depth > heading.depth);
    let end = |index: usize| {
        outline
            .blocks
            .get(index + 1)
            .map_or(blocks.len(), |next| next.index)
    };
    for (offset, block) in descendants.enumerate() {
        let range = block.index..end(first + 1 + offset);
        if let Some(destination) = first_audio(&blocks[range], 0) {
            return Some(Recording { destination, title });
        }
    }
    None
}

/// Whether a destination names an audio file the published page plays: `.mp3`, `.wav` or
/// `.ogg`, ignoring a query or fragment.
fn is_audio(destination: &str) -> bool {
    let path = destination.split(['?', '#']).next().unwrap_or(destination);
    path.rsplit_once('.').is_some_and(|(_, extension)| {
        matches!(
            extension.to_ascii_lowercase().as_str(),
            "mp3" | "wav" | "ogg"
        )
    })
}

fn first_audio(blocks: &[Block], depth: usize) -> Option<String> {
    if depth > MAX_DEPTH {
        return None;
    }
    blocks.iter().find_map(|block| match block {
        Block::Paragraph { inline, .. }
        | Block::Heading { inline, .. }
        | Block::Bullet { inline, .. } => audio_in(inline, depth + 1),
        Block::Quote { children, .. } | Block::Custom { children, .. } => {
            first_audio(children, depth + 1)
        }
        Block::List { items, .. } => audio_in_items(items, depth + 1),
        _ => None,
    })
}

fn audio_in_items(items: &[ListItem], depth: usize) -> Option<String> {
    if depth > MAX_DEPTH {
        return None;
    }
    items.iter().find_map(|item| {
        audio_in(&item.name, depth + 1)
            .or_else(|| first_audio(&item.content, depth + 1))
            .or_else(|| audio_in_items(&item.items, depth + 1))
    })
}

fn audio_in(nodes: &[Inline], depth: usize) -> Option<String> {
    if depth > MAX_DEPTH {
        return None;
    }
    nodes.iter().find_map(|node| match node {
        Inline::Link {
            url, image: true, ..
        } => destination(url).filter(|destination| is_audio(destination)),
        Inline::Emphasis { children, .. }
        | Inline::Subscript { children, .. }
        | Inline::Superscript { children, .. } => audio_in(children, depth + 1),
        _ => None,
    })
}

/// A link's destination as the renderer reads it; `None` for page and block references.
fn destination(url: &Url) -> Option<String> {
    match url {
        Url::File { v } | Url::Search { v } | Url::EmbedData { v } => Some(v.clone()),
        Url::Complex { protocol, link } => Some(match (protocol, link) {
            (Some(protocol), Some(link)) => format!("{protocol}://{link}"),
            (_, link) => link.clone().unwrap_or_default(),
        }),
        Url::PageRef { .. } | Url::BlockRef { .. } => None,
    }
}

/// The text a reader sees in `nodes`: page references by name, links by label.
fn plain_text(nodes: &[Inline], out: &mut String, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    for node in nodes {
        match node {
            Inline::Plain { text, .. }
            | Inline::Code { text, .. }
            | Inline::Verbatim { text, .. } => {
                out.push_str(text);
            }
            Inline::Break { .. } | Inline::HardBreak { .. } => out.push(' '),
            Inline::Emphasis { children, .. }
            | Inline::Subscript { children, .. }
            | Inline::Superscript { children, .. }
            | Inline::Tag { children, .. } => plain_text(children, out, depth + 1),
            Inline::Link { label, .. } if !label.is_empty() => plain_text(label, out, depth + 1),
            Inline::Link {
                url: Url::PageRef { v },
                ..
            } => out.push_str(v),
            Inline::Link { url, .. } => out.push_str(&destination(url).unwrap_or_default()),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(source: &str) -> Option<Recording> {
        recording(&lsdoc::parse_to_projection(source).blocks)
    }

    #[test]
    fn recording_under_the_first_heading() {
        let page = "prev:: [[A]]\nlogseq-entity:: [[Logseq/Entity/Podcast/Episode]]\n\n- # The Unstable over USB\n\t- A session at the library.\n\t- ![Episode recording](https://example.com/24-Sat.mp3)\n\t- ## Recording\n\t\t- ![Second](https://example.com/second.mp3)\n";
        assert_eq!(
            found(page),
            Some(Recording {
                destination: "https://example.com/24-Sat.mp3".to_owned(),
                title: "The Unstable over USB".to_owned(),
            })
        );
    }

    #[test]
    fn older_shape_with_a_local_asset() {
        let page = "logseq-entity:: [[Logseq/Entity/Podcast/Episode]]\n- # GitP.26.09.24\n\t- A MicroFreak session.\n\t- ## Notes\n\t  collapsed:: true\n\t\t- ![Synth](../assets/Synth___Full.mp3)\n";
        let recording = found(page).unwrap();
        assert_eq!(recording.destination, "../assets/Synth___Full.mp3");
        assert_eq!(recording.title, "GitP.26.09.24");
    }

    #[test]
    fn audio_outside_the_first_heading_block_is_not_the_recording() {
        let page = "- # Episode\n\t- No audio here.\n- ![Elsewhere](https://example.com/elsewhere.mp3)\n- # Second heading\n\t- ![Later](https://example.com/later.mp3)\n";
        assert_eq!(found(page), None);
        let before = "- ![Before](https://example.com/before.mp3)\n- # Episode\n\t- text\n";
        assert_eq!(found(before), None);
    }

    #[test]
    fn page_without_a_recording() {
        assert_eq!(found("- # Ghost in the Patch\n\t- GitHub repo\n"), None);
        assert_eq!(found("- Just text ![x](https://example.com/a.mp3)\n"), None);
        // An image or a plain link to audio is not an embed.
        assert_eq!(
            found(
                "- # E\n\t- ![photo](https://example.com/a.png)\n\t- [audio](https://example.com/a.mp3)\n"
            ),
            None
        );
    }

    #[test]
    fn title_reads_references_and_emphasis_as_text() {
        let page = "- # Infini Pad and [[Microfreak/Preset/260 8bitfi]] **live**\n\t- ![r](https://example.com/r.mp3?x=1)\n";
        let recording = found(page).unwrap();
        assert_eq!(
            recording.title,
            "Infini Pad and Microfreak/Preset/260 8bitfi live"
        );
        assert_eq!(recording.destination, "https://example.com/r.mp3?x=1");
    }
}
