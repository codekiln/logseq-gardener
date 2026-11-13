//! A podcast episode page: the recording it plays, the title it shows and its chapters.
//!
//! A page is an episode when its `logseq-entity::` value names [`EPISODE_TYPE`] (see
//! [`crate::entity_types`]). Its recording is the first audio embed, `![label](url.mp3)`,
//! among the descendants of the page's first heading block, and its title is that heading's
//! text. Its chapters are the H3 to H6 headings that start with a timestamp, and the bullets
//! under its `## Chapters` heading that start with one ([`chapters`]).
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

/// One chapter of an episode: an H3 to H6 heading, or a bullet under `## Chapters`, whose text
/// starts with a timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chapter {
    /// Byte offset of the chapter's bullet in the page source, which the renderer anchors as
    /// `source-block-<offset>`.
    pub start: usize,
    /// Where the chapter starts in the recording.
    pub seconds: u32,
    /// The first line's text after the timestamp and any dash that follows it.
    pub title: String,
    /// Index in the returned list of the nearest chapter among the bullet's ancestors.
    pub parent: Option<usize>,
}

/// The chapters of a page's outline in page order: each H3 to H6 heading, and each bullet at
/// any depth under a `## Chapters` heading, whose text starts with a `M:SS`, `MM:SS` or
/// `H:MM:SS` timestamp (see [`leading_timestamp`]). A timestamp later in the block, or one
/// inside code or emphasis, leaves the block as text.
///
/// ```
/// use logseq_gardener_sdk::podcast::chapters;
///
/// let parsed = lsdoc::parse_to_projection(
///     "- # Episode\n\t- ## Chapters\n\t\t- ### 18:17 Enter 8bitfi\n\t\t\t- 19:30 - Old warbling\n",
/// );
/// let found = chapters(&parsed.blocks);
/// assert_eq!(found[0].seconds, 1097);
/// assert_eq!(found[1].title, "Old warbling");
/// assert_eq!(found[1].parent, Some(0));
/// ```
pub fn chapters(blocks: &[Block]) -> Vec<Chapter> {
    let outline = Outline::from_blocks(blocks);
    let mut found = Vec::new();
    // The chapter each outline block opens, by outline index.
    let mut opened: Vec<Option<usize>> = Vec::with_capacity(outline.blocks.len());
    // Whether each outline block is a descendant of a `## Chapters` heading, by outline index.
    let mut listed: Vec<bool> = Vec::with_capacity(outline.blocks.len());
    for block in &outline.blocks {
        let under_chapters = block.parent.is_some_and(|parent| {
            listed[parent] || is_chapters_heading(blocks, &outline.blocks[parent])
        });
        listed.push(under_chapters);
        let candidate = under_chapters || matches!(block.heading, Some(3..=6));
        let chapter = candidate.then(|| chapter_of(blocks, block)).flatten().map(
            |(start, seconds, title)| {
                let parent =
                    std::iter::successors(block.parent, |&index| outline.blocks[index].parent)
                        .find_map(|index| opened[index]);
                found.push(Chapter {
                    start,
                    seconds,
                    title,
                    parent,
                });
                found.len() - 1
            },
        );
        opened.push(chapter);
    }
    found
}

/// Whether `block` is an H2 heading whose text is `Chapters`, ignoring case and surrounding
/// whitespace.
fn is_chapters_heading(blocks: &[Block], block: &crate::outline::OutlineBlock) -> bool {
    if block.heading != Some(2) {
        return false;
    }
    let Block::Bullet { inline, .. } = &blocks[block.index] else {
        return false;
    };
    let mut text = String::new();
    plain_text(inline, &mut text, 0);
    text.trim().eq_ignore_ascii_case("Chapters")
}

/// `(start, seconds, title)` when the text of `block` starts with a timestamp.
fn chapter_of(
    blocks: &[Block],
    block: &crate::outline::OutlineBlock,
) -> Option<(usize, u32, String)> {
    let start = block.start?;
    // A task marker or a priority comes before the text, so the text does not start the block.
    let Block::Bullet {
        inline,
        marker: None,
        priority: None,
        ..
    } = &blocks[block.index]
    else {
        return None;
    };
    let Some(Inline::Plain { text, .. }) = inline.first() else {
        return None;
    };
    let (seconds, length) = leading_timestamp(text)?;
    let first_line = inline
        .iter()
        .position(|node| matches!(node, Inline::Break { .. } | Inline::HardBreak { .. }))
        .map_or(&inline[..], |end| &inline[..end]);
    let mut title = String::new();
    title.push_str(&text[length..]);
    plain_text(&first_line[1..], &mut title, 0);
    let title = title
        .trim_start_matches(|c: char| c.is_whitespace() || matches!(c, '-' | '–' | '—'))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    Some((start, seconds, title))
}

/// The timestamp that starts `text` as `(seconds, byte length)`: one or two digits, then one
/// or two groups of a colon and two digits under 60, so `5:15`, `18:17`, `1:02:03` and
/// `01:00:23`. The timestamp must end the text or be followed by whitespace or a dash, so
/// `18:17pm` and `18:170` are not timestamps.
///
/// ```
/// use logseq_gardener_sdk::podcast::leading_timestamp;
///
/// assert_eq!(leading_timestamp("18:17 Enter"), Some((1097, 5)));
/// assert_eq!(leading_timestamp("01:00:23"), Some((3623, 8)));
/// assert_eq!(leading_timestamp("at 18:17"), None);
/// ```
#[must_use]
pub fn leading_timestamp(text: &str) -> Option<(u32, usize)> {
    let bytes = text.as_bytes();
    let digits = bytes.iter().take_while(|b| b.is_ascii_digit()).count();
    if !(1..=2).contains(&digits) {
        return None;
    }
    let mut value: u32 = text[..digits].parse().ok()?;
    let mut length = digits;
    let mut groups = 0;
    while groups < 2
        && let [b':', tens @ b'0'..=b'5', ones @ b'0'..=b'9', ..] = bytes[length..]
    {
        value = value * 60 + u32::from(tens - b'0') * 10 + u32::from(ones - b'0');
        length += 3;
        groups += 1;
    }
    let ends = text[length..]
        .chars()
        .next()
        .is_none_or(|c| c.is_whitespace() || matches!(c, '-' | '–' | '—'));
    (groups > 0 && ends).then_some((value, length))
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
pub(crate) fn plain_text(nodes: &[Inline], out: &mut String, depth: usize) {
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

    fn chapters_of(source: &str) -> Vec<(u32, String, Option<usize>)> {
        chapters(&lsdoc::parse_to_projection(source).blocks)
            .into_iter()
            .map(|chapter| (chapter.seconds, chapter.title, chapter.parent))
            .collect()
    }

    #[test]
    fn timestamp_formats() {
        assert_eq!(leading_timestamp("5:15"), Some((315, 4)));
        assert_eq!(leading_timestamp("05:15 - the bass"), Some((315, 5)));
        assert_eq!(leading_timestamp("18:17 Enter"), Some((1097, 5)));
        assert_eq!(leading_timestamp("1:02:03 An hour in"), Some((3723, 7)));
        assert_eq!(leading_timestamp("01:00:23 Recording"), Some((3623, 8)));
        assert_eq!(leading_timestamp("13:41 – em dash"), Some((821, 5)));
        assert_eq!(leading_timestamp("13:41—tight"), Some((821, 5)));
        for text in [
            "",
            "18",
            "18:",
            "18:7",
            "18:60",
            "123:00",
            "18:17pm",
            "18:170",
            "1:02:03:04",
            "18:17:6",
            " 18:17",
            "at 18:17",
            "Layer C capture",
        ] {
            assert_eq!(leading_timestamp(text), None, "{text:?}");
        }
    }

    #[test]
    fn chapter_headings_at_each_level() {
        let page = "- # Episode\n\t- ![r](https://example.com/r.mp3)\n\t- ## 00:00 An H2 is not a chapter\n\t- ## Chapters\n\t\t- ### 00:00 Enter [[Microfreak/Preset/259 Infini Pad]]\n\t\t\t- #### 00:00 [[Launchpad]] playing\n\t\t\t- #### 05:12 Infini Pad as bass\n\t\t\t\t- Notes, not a chapter.\n\t\t\t\t\t- ##### 13:41 - pointillist emergence\n\t\t\t\t\t\t- ###### 1:02:03 deepest\n\t\t- ### 18:17 Enter 8bitfi\n";
        assert_eq!(
            chapters_of(page),
            [
                (0, "Enter Microfreak/Preset/259 Infini Pad".to_owned(), None),
                (0, "Launchpad playing".to_owned(), Some(0)),
                (312, "Infini Pad as bass".to_owned(), Some(0)),
                (821, "pointillist emergence".to_owned(), Some(2)),
                (3723, "deepest".to_owned(), Some(3)),
                (1097, "Enter 8bitfi".to_owned(), None),
            ]
        );
    }

    #[test]
    fn timestamp_in_code_or_later_in_the_heading_is_text() {
        let page = "- ### `05:15` in code\n- ### the second voice joins at 01:28\n- ### **05:15** emphasised\n- ### Layer C capture\n\t- #### 05:20 Under a plain heading\n- 05:30 a bullet is not a heading chapter\n- ~~~\n  ### 05:40 in a code block\n  ~~~\n";
        assert_eq!(
            chapters_of(page),
            [(320, "Under a plain heading".to_owned(), None)]
        );
    }

    #[test]
    fn two_headings_at_the_same_time_are_two_chapters_with_their_own_anchors() {
        let page =
            "- ### 00:00 Enter Voidom\n\t- #### 00:00 tentative invocation\n- ### 00:00 Again\n";
        let found = chapters(&lsdoc::parse_to_projection(page).blocks);
        assert_eq!(found.len(), 3);
        assert!(found.iter().all(|chapter| chapter.seconds == 0));
        assert_eq!(found[1].parent, Some(0));
        assert_eq!(found[2].parent, None);
        let starts: std::collections::BTreeSet<_> = found.iter().map(|c| c.start).collect();
        assert_eq!(starts.len(), 3);
        assert_eq!(&page[found[2].start..found[2].start + 1], "-");
    }

    #[test]
    fn timestamp_led_bullets_under_chapters_at_any_depth() {
        let page = "- # Episode\n\t- ![r](https://example.com/r.mp3)\n\t- ## Chapters\n\t\t- ### 00:00 enter [[Microfreak/Preset/016 Voidom]]\n\t\t\t- #### 03:07 secondo strato\n\t\t\t\t- layer B enters while layer A loops.\n\t\t\t- [[My Notes]]\n\t\t\t\t- 05:15 - the bass comes in\n\t\t\t\t\t- 05:40 - and stays\n\t\t\t\t\t\t- 1:02:03 deeper than any heading\n\t\t- 18:17 — Let's try this 8bitfi lead, 260.\n";
        assert_eq!(
            chapters_of(page),
            [
                (0, "enter Microfreak/Preset/016 Voidom".to_owned(), None),
                (187, "secondo strato".to_owned(), Some(0)),
                (315, "the bass comes in".to_owned(), Some(0)),
                (340, "and stays".to_owned(), Some(2)),
                (3723, "deeper than any heading".to_owned(), Some(3)),
                (1097, "Let's try this 8bitfi lead, 260.".to_owned(), None),
            ]
        );
    }

    #[test]
    fn mid_text_code_and_task_bullets_under_chapters_are_text() {
        let page = "- ## Chapters\n\t- ### 01:00 Opening\n\t\t- `05:15` in code\n\t\t- the second voice joins at 01:28\n\t\t- **05:15** emphasised\n\t\t- TODO 05:20 check the take\n\t\t- [#A] 05:25 with a priority\n\t\t- ~~~\n\t\t  05:30 in a code block\n\t\t  ~~~\n\t\t- 05:35pm is not a timestamp\n";
        assert_eq!(chapters_of(page), [(60, "Opening".to_owned(), None)]);
    }

    #[test]
    fn timestamp_led_bullets_outside_chapters_are_text() {
        let page = "- # Episode\n\t- 00:10 under the title\n\t- ## Notes\n\t\t- 00:20 under another heading\n\t- ## Chapters\n\t\t- 00:30 listed\n\t- Chapters\n\t\t- 00:40 under a plain bullet named Chapters\n\t- ### Chapters\n\t\t- 00:50 under an H3 named Chapters\n- 01:00 at the top level\n";
        assert_eq!(chapters_of(page), [(30, "listed".to_owned(), None)]);
        // The heading text is matched ignoring case, and a `heading:: 2` property counts.
        let property = "- chapters\n  heading:: 2\n\t- 00:30 listed\n";
        assert_eq!(chapters_of(property), [(30, "listed".to_owned(), None)]);
    }

    #[test]
    fn heading_property_and_a_second_line() {
        let page =
            "- 02:00 Set by property\n  heading:: 3\n- ### 03:00 First line\n  continued below\n";
        assert_eq!(
            chapters_of(page),
            [
                (120, "Set by property".to_owned(), None),
                (180, "First line".to_owned(), None),
            ]
        );
    }
}
