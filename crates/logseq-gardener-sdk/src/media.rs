//! Video macros, YouTube timestamps, and which video each timestamp seeks.
//!
//! Follows Logseq's `get-matched-video` and `parse-timestamp`, with two additions: a
//! start time also comes from `?t=` and `1m30s` forms, and timestamps bind to the
//! nearest YouTube video before them in their page's outline.
use std::collections::BTreeMap;

use crate::ast::{Block, Inline, ListItem};

const MAX_DEPTH: usize = 64;

/// Who plays a video, which decides its embed URL and whether timestamps can seek it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    YouTube,
    YouTubeNoCookie,
    Loom,
    Vimeo,
    Bilibili,
    /// Any other URL, embedded as written.
    Other,
}

impl Provider {
    /// The provider's name for accessible labels and iframe titles.
    pub fn name(self) -> &'static str {
        match self {
            Self::YouTube | Self::YouTubeNoCookie => "YouTube",
            Self::Loom => "Loom",
            Self::Vimeo => "Vimeo",
            Self::Bilibili => "Bilibili",
            Self::Other => "Embedded",
        }
    }
    /// The lower-case key a page's script reads.
    pub fn key(self) -> &'static str {
        match self {
            Self::YouTube => "youtube",
            Self::YouTubeNoCookie => "youtube-nocookie",
            Self::Loom => "loom",
            Self::Vimeo => "vimeo",
            Self::Bilibili => "bilibili",
            Self::Other => "other",
        }
    }
    /// Whether a YouTube timestamp can seek this video.
    pub fn seekable(self) -> bool {
        matches!(self, Self::YouTube | Self::YouTubeNoCookie)
    }
}

/// A playable video from a video macro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoEmbed {
    pub provider: Provider,
    pub video_id: Option<String>,
    pub embed_url: String,
    pub watch_url: String,
    /// Seconds into the video where playback starts.
    pub start: Option<u64>,
}

/// What a video macro's argument turned out to be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoArgument {
    Video(VideoEmbed),
    /// Not a URL (`{{video}}`), or no recognisable video (`{{youtube}}` and others).
    Invalid,
    Empty,
}

/// Whether `name` is a macro that embeds a video.
pub fn is_video_macro(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "video" | "youtube" | "vimeo" | "bilibili"
    )
}

/// Whether `name` is the YouTube timestamp macro.
pub fn is_timestamp_macro(name: &str) -> bool {
    name.eq_ignore_ascii_case("youtube-timestamp")
}

/// Classifies a video macro's first argument; `None` when `name` is not a video macro.
pub fn video(name: &str, args: &[String]) -> Option<VideoArgument> {
    let name = name.to_ascii_lowercase();
    if !is_video_macro(&name) {
        return None;
    }
    let Some(argument) = args.first().map(|arg| arg.trim()).filter(|a| !a.is_empty()) else {
        return Some(VideoArgument::Empty);
    };
    let embed = match name.as_str() {
        "video" => {
            if !is_url(argument) {
                return Some(VideoArgument::Invalid);
            }
            Some(classify(argument))
        }
        "youtube" => {
            let id = if argument.chars().count() == 11 {
                Some(argument.to_owned())
            } else {
                youtube_match(argument).map(|(_, id, _)| id)
            };
            id.filter(|id| !id.is_empty())
                .map(|id| youtube(Provider::YouTube, &id, None))
        }
        "vimeo" => short_id(argument)
            .or_else(|| vimeo_match(argument).map(|(id, _)| id))
            .map(|id| vimeo(&id)),
        _ => short_id(argument)
            .or_else(|| bilibili_match(argument).map(|(id, _, _)| id))
            .map(|id| bilibili(&id, None)),
    };
    Some(embed.map_or(VideoArgument::Invalid, VideoArgument::Video))
}

fn short_id(argument: &str) -> Option<String> {
    (argument.chars().count() <= 15).then(|| argument.to_owned())
}

/// Logseq's `url?`: a scheme, `://` and a host, with no whitespace.
fn is_url(text: &str) -> bool {
    let Some((scheme, rest)) = text.split_once("://") else {
        return false;
    };
    !scheme.is_empty()
        && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
        && !rest
            .split(['/', '?', '#'])
            .next()
            .unwrap_or_default()
            .is_empty()
        && !text.chars().any(char::is_whitespace)
}

/// Follows `get-matched-video`: YouTube, Loom, Vimeo, then Bilibili.
fn classify(url: &str) -> VideoEmbed {
    if let Some((host, id, _)) = youtube_match(url) {
        if host == "youtube-nocookie.com" {
            return youtube(Provider::YouTubeNoCookie, &id, start_time(url));
        }
        if id.len() == 11 {
            return youtube(Provider::YouTube, &id, start_time(url));
        }
        return other(url);
    }
    if let Some(id) = loom_match(url) {
        return VideoEmbed {
            provider: Provider::Loom,
            embed_url: format!("https://www.loom.com/embed/{id}"),
            watch_url: format!("https://www.loom.com/share/{id}"),
            video_id: Some(id),
            start: None,
        };
    }
    if let Some((id, _)) = vimeo_match(url) {
        return vimeo(&id);
    }
    if let Some((id, page, _)) = bilibili_match(url) {
        return bilibili(&id, page);
    }
    other(url)
}

fn other(url: &str) -> VideoEmbed {
    VideoEmbed {
        provider: Provider::Other,
        video_id: None,
        embed_url: url.to_owned(),
        watch_url: url.to_owned(),
        start: None,
    }
}

fn youtube(provider: Provider, id: &str, start: Option<u64>) -> VideoEmbed {
    let embed_url = match provider {
        Provider::YouTubeNoCookie => format!("https://www.youtube-nocookie.com/embed/{id}"),
        _ => format!("https://www.youtube.com/embed/{id}?enablejsapi=1"),
    };
    VideoEmbed {
        provider,
        video_id: Some(id.to_owned()),
        embed_url,
        watch_url: format!("https://www.youtube.com/watch?v={id}"),
        start,
    }
}

fn vimeo(id: &str) -> VideoEmbed {
    VideoEmbed {
        provider: Provider::Vimeo,
        video_id: Some(id.to_owned()),
        embed_url: format!("https://player.vimeo.com/video/{id}"),
        watch_url: format!("https://vimeo.com/{id}"),
        start: None,
    }
}

fn bilibili(id: &str, page: Option<String>) -> VideoEmbed {
    let page_query = page
        .as_ref()
        .map(|p| format!("&page={p}"))
        .unwrap_or_default();
    VideoEmbed {
        provider: Provider::Bilibili,
        video_id: Some(id.to_owned()),
        embed_url: format!(
            "https://player.bilibili.com/player.html?bvid={id}&high_quality=1&autoplay=0{page_query}"
        ),
        watch_url: match page {
            Some(page) => format!("https://www.bilibili.com/video/{id}?p={page}"),
            None => format!("https://www.bilibili.com/video/{id}"),
        },
        start: None,
    }
}

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// The leading `[\w-]+` of `text` and what follows it.
fn word(text: &str) -> Option<(&str, &str)> {
    let end = text.find(|c: char| !is_word(c)).unwrap_or(text.len());
    (end > 0).then(|| text.split_at(end))
}

/// Strips `https://`, `http://` or `//`, then an optional one-character-separated
/// `www` (or `m` when `mobile`), and matches one of `hosts`; the regexes' `.` matches
/// any character, as here.
fn host<'a>(url: &'a str, hosts: &[&'static str], mobile: bool) -> Option<(&'static str, &'a str)> {
    if url.chars().any(char::is_whitespace) {
        return None;
    }
    let lower = url.to_ascii_lowercase();
    let mut rest = url;
    for scheme in ["https://", "http://", "//"] {
        if lower.starts_with(scheme) {
            rest = &url[scheme.len()..];
            break;
        }
    }
    let mut candidates = vec![rest];
    for prefix in ["www", "m"] {
        if (prefix == "www" || mobile) && rest.starts_with(prefix) {
            let after = &rest[prefix.len()..];
            if let Some(c) = after.chars().next() {
                candidates.push(&after[c.len_utf8()..]);
            }
        }
    }
    for candidate in candidates.into_iter().rev() {
        for host in hosts {
            if let Some(after) = candidate.strip_prefix(host) {
                return Some((host, after));
            }
        }
    }
    None
}

/// `youtube-regex`: the host, the id and the rest of the URL.
fn youtube_match(url: &str) -> Option<(&'static str, String, String)> {
    let (host, after) = host(
        url,
        &["youtube.com", "youtu.be", "y2u.be", "youtube-nocookie.com"],
        true,
    )?;
    let path = after.strip_prefix('/').unwrap_or(after);
    let path = if after.starts_with('/') {
        if let Some(rest) = path
            .strip_prefix("embed/")
            .or_else(|| path.strip_prefix("v/"))
        {
            rest
        } else if let Some((name, rest)) = word(path)
            && let Some(rest) = rest.strip_prefix("?v=")
            && !name.is_empty()
        {
            rest
        } else {
            path
        }
    } else {
        after
    };
    let (id, rest) = word(path)?;
    Some((host, id.to_owned(), rest.to_owned()))
}

/// `loom-regex`: the id after `/share/` or `/embed/`.
fn loom_match(url: &str) -> Option<String> {
    let (_, after) = host(url, &["loom.com"], false)?;
    let path = after
        .strip_prefix("/share/")
        .or_else(|| after.strip_prefix("/embed/"))?;
    word(path).map(|(id, _)| id.to_owned())
}

/// `vimeo-regex`: the id after `/` or `/video/`.
fn vimeo_match(url: &str) -> Option<(String, String)> {
    let (_, after) = host(url, &["player.vimeo.com", "vimeo.com"], false)?;
    let path = after.strip_prefix('/')?;
    let path = path.strip_prefix("video/").unwrap_or(path);
    word(path).map(|(id, rest)| (id.to_owned(), rest.to_owned()))
}

/// `bilibili-regex`: the id, the `?p=` page and the rest.
fn bilibili_match(url: &str) -> Option<(String, Option<String>, String)> {
    let (_, after) = host(url, &["bilibili.com"], false)?;
    let path = after.strip_prefix('/')?;
    let path = path.strip_prefix("video/").unwrap_or(path);
    let (id, rest) = word(path)?;
    let page = rest.strip_prefix("?p=").and_then(|p| {
        let digits: String = p.chars().take_while(char::is_ascii_digit).collect();
        (!digits.is_empty()).then_some(digits)
    });
    Some((id.to_owned(), page, rest.to_owned()))
}

/// `t=` from the query or the fragment: `205`, `205s` or `1h2m3s`.
fn start_time(url: &str) -> Option<u64> {
    let (before_fragment, fragment) = url.split_once('#').unwrap_or((url, ""));
    let query = before_fragment.split_once('?').map_or("", |(_, q)| q);
    query
        .split('&')
        .chain(fragment.split('&'))
        .filter_map(|pair| pair.strip_prefix("t="))
        .find_map(duration)
}

fn duration(value: &str) -> Option<u64> {
    let value = value.trim();
    if !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()) {
        return value.parse().ok();
    }
    let mut total = 0_u64;
    let mut digits = String::new();
    let mut seen = false;
    let mut last_unit = 0;
    for c in value.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
            continue;
        }
        let (unit, factor) = match c.to_ascii_lowercase() {
            'h' => (1, 3600),
            'm' => (2, 60),
            's' => (3, 1),
            _ => return None,
        };
        if digits.is_empty() || unit <= last_unit {
            return None;
        }
        total = total.checked_add(digits.parse::<u64>().ok()?.checked_mul(factor)?)?;
        digits.clear();
        last_unit = unit;
        seen = true;
    }
    (seen && digits.is_empty()).then_some(total)
}

/// A point in a YouTube video from `{{youtube-timestamp N}}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp {
    pub seconds: u64,
}

impl Timestamp {
    /// Whole seconds, `M:SS` or `H:MM:SS` with minutes and seconds 0–59, as
    /// Logseq's `parse-timestamp` accepts.
    pub fn parse(argument: &str) -> Option<Self> {
        let argument = argument.trim();
        if argument.is_empty() {
            return None;
        }
        if argument.bytes().all(|b| b.is_ascii_digit()) {
            return argument.parse().ok().map(|seconds| Self { seconds });
        }
        let parts: Vec<&str> = argument.split(':').collect();
        if !(2..=3).contains(&parts.len()) {
            return None;
        }
        let mut values = Vec::new();
        for (index, part) in parts.iter().enumerate() {
            let sixty = index + 2 >= parts.len();
            if part.is_empty()
                || !part.bytes().all(|b| b.is_ascii_digit())
                || (sixty && part.len() > 2)
            {
                return None;
            }
            let value: u64 = part.parse().ok()?;
            if sixty && value > 59 {
                return None;
            }
            values.push(value);
        }
        let seconds = values.iter().try_fold(0_u64, |total, value| {
            total.checked_mul(60)?.checked_add(*value)
        })?;
        Some(Self { seconds })
    }
    /// From a macro's arguments: the first one.
    pub fn from_args(args: &[String]) -> Option<Self> {
        args.first().and_then(|arg| Self::parse(arg))
    }
    /// `MM:SS`, or `HH:MM:SS` when the hour is not zero, as Logseq's `seconds->display`.
    pub fn display(self) -> String {
        let hours = self.seconds / 3600;
        let minutes = (self.seconds / 60) % 60;
        let seconds = self.seconds % 60;
        if hours == 0 {
            format!("{minutes:02}:{seconds:02}")
        } else {
            format!("{hours:02}:{minutes:02}:{seconds:02}")
        }
    }
}

/// A watch URL at `seconds`, for a timestamp's link.
pub fn watch_at(video: &VideoEmbed, seconds: u64) -> String {
    let separator = if video.watch_url.contains('?') {
        '&'
    } else {
        '?'
    };
    format!("{}{separator}t={seconds}s", video.watch_url)
}

/// A page's videos in outline order and the video each timestamp seeks, keyed by the
/// byte offset where the macro starts in the page's source.
#[derive(Debug, Default, Clone)]
pub struct PageMedia {
    pub videos: Vec<VideoEmbed>,
    video_at: BTreeMap<usize, usize>,
    timestamp_at: BTreeMap<usize, (Timestamp, Option<usize>)>,
}

impl PageMedia {
    /// Walks `blocks` depth first, parents before children, numbering each valid video
    /// and binding each timestamp to the nearest YouTube video before it.
    pub fn of(blocks: &[Block]) -> Self {
        let mut media = Self::default();
        let mut last_youtube = None;
        media.walk_blocks(blocks, &mut last_youtube, 0);
        media
    }
    /// The number and video of the video macro starting at `offset`.
    pub fn video_at(&self, offset: usize) -> Option<(usize, &VideoEmbed)> {
        let index = *self.video_at.get(&offset)?;
        Some((index, &self.videos[index]))
    }
    /// The timestamp starting at `offset` and the number of the video it seeks.
    pub fn timestamp_at(&self, offset: usize) -> Option<(Timestamp, Option<usize>)> {
        self.timestamp_at.get(&offset).copied()
    }
    /// Every timestamp in outline order with the video it seeks.
    pub fn timestamps(&self) -> impl Iterator<Item = (Timestamp, Option<&VideoEmbed>)> {
        self.timestamp_at
            .values()
            .map(|(time, video)| (*time, video.map(|index| &self.videos[index])))
    }
    fn walk_blocks(&mut self, blocks: &[Block], last: &mut Option<usize>, depth: usize) {
        if depth > MAX_DEPTH {
            return;
        }
        for block in blocks {
            match block {
                Block::Paragraph { inline, .. }
                | Block::Heading { inline, .. }
                | Block::Bullet { inline, .. } => self.walk_inlines(inline, last, depth + 1),
                Block::Quote { children, .. } | Block::Custom { children, .. } => {
                    self.walk_blocks(children, last, depth + 1)
                }
                Block::List { items, .. } => self.walk_items(items, last, depth + 1),
                _ => {}
            }
        }
    }
    fn walk_items(&mut self, items: &[ListItem], last: &mut Option<usize>, depth: usize) {
        if depth > MAX_DEPTH {
            return;
        }
        for item in items {
            self.walk_inlines(&item.name, last, depth + 1);
            self.walk_blocks(&item.content, last, depth + 1);
            self.walk_items(&item.items, last, depth + 1);
        }
    }
    fn walk_inlines(&mut self, nodes: &[Inline], last: &mut Option<usize>, depth: usize) {
        if depth > MAX_DEPTH {
            return;
        }
        for node in nodes {
            match node {
                Inline::Macro { name, args, span } => {
                    let Some(offset) = span.map(|s| s.0) else {
                        continue;
                    };
                    if let Some(VideoArgument::Video(embed)) = video(name, args) {
                        let index = self.videos.len();
                        if embed.provider.seekable() {
                            *last = Some(index);
                        }
                        self.videos.push(embed);
                        self.video_at.insert(offset, index);
                    } else if is_timestamp_macro(name)
                        && let Some(time) = Timestamp::from_args(args)
                    {
                        self.timestamp_at.insert(offset, (time, *last));
                    }
                }
                Inline::Emphasis { children, .. }
                | Inline::Subscript { children, .. }
                | Inline::Superscript { children, .. }
                | Inline::Tag { children, .. } => self.walk_inlines(children, last, depth + 1),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn embed(name: &str, arg: &str) -> VideoEmbed {
        match video(name, &[arg.to_owned()]) {
            Some(VideoArgument::Video(embed)) => embed,
            other => panic!("{name} {arg}: {other:?}"),
        }
    }

    #[test]
    fn classifies_every_provider() {
        let cases = [
            (
                "https://www.youtube.com/watch?v=6cqVzgVQJfE",
                Provider::YouTube,
                "https://www.youtube.com/embed/6cqVzgVQJfE?enablejsapi=1",
            ),
            (
                "https://youtu.be/yxi1O0T4l6k?t=14",
                Provider::YouTube,
                "https://www.youtube.com/embed/yxi1O0T4l6k?enablejsapi=1",
            ),
            (
                "https://y2u.be/6cqVzgVQJfE",
                Provider::YouTube,
                "https://www.youtube.com/embed/6cqVzgVQJfE?enablejsapi=1",
            ),
            (
                "https://m.youtube.com/watch?v=6cqVzgVQJfE",
                Provider::YouTube,
                "https://www.youtube.com/embed/6cqVzgVQJfE?enablejsapi=1",
            ),
            (
                "https://www.youtube.com/embed/6cqVzgVQJfE",
                Provider::YouTube,
                "https://www.youtube.com/embed/6cqVzgVQJfE?enablejsapi=1",
            ),
            (
                "https://www.youtube.com/shorts/abc",
                Provider::Other,
                "https://www.youtube.com/shorts/abc",
            ),
            (
                "https://www.youtube-nocookie.com/embed/6cqVzgVQJfE",
                Provider::YouTubeNoCookie,
                "https://www.youtube-nocookie.com/embed/6cqVzgVQJfE",
            ),
            (
                "https://www.loom.com/share/0281766fa2d04bb788eaf19e65135184",
                Provider::Loom,
                "https://www.loom.com/embed/0281766fa2d04bb788eaf19e65135184",
            ),
            (
                "https://vimeo.com/76979871",
                Provider::Vimeo,
                "https://player.vimeo.com/video/76979871",
            ),
            (
                "https://player.vimeo.com/video/76979871",
                Provider::Vimeo,
                "https://player.vimeo.com/video/76979871",
            ),
            (
                "https://www.bilibili.com/video/BV1GJ411x7h7?p=2",
                Provider::Bilibili,
                "https://player.bilibili.com/player.html?bvid=BV1GJ411x7h7&high_quality=1&autoplay=0&page=2",
            ),
            (
                "https://example.com/film.mp4",
                Provider::Other,
                "https://example.com/film.mp4",
            ),
        ];
        for (url, provider, embed_url) in cases {
            let embed = embed("video", url);
            assert_eq!(embed.provider, provider, "{url}");
            assert_eq!(embed.embed_url, embed_url, "{url}");
        }
    }

    #[test]
    fn reads_start_times_from_query_or_fragment() {
        let start = |url| embed("video", url).start;
        assert_eq!(start("https://youtu.be/yxi1O0T4l6k?t=14"), Some(14));
        assert_eq!(
            start("https://www.youtube.com/watch?v=TQsP_PlCY1I&t=205s"),
            Some(205)
        );
        assert_eq!(
            start("https://www.youtube.com/watch?v=TQsP_PlCY1I#t=1m30s"),
            Some(90)
        );
        assert_eq!(
            start("https://www.youtube.com/watch?v=TQsP_PlCY1I&t=1h2m3s"),
            Some(3723)
        );
        assert_eq!(start("https://www.youtube.com/watch?v=TQsP_PlCY1I"), None);
        assert_eq!(
            start("https://www.youtube.com/watch?v=TQsP_PlCY1I&t=x"),
            None
        );
        assert_eq!(
            embed(
                "video",
                "https://www.youtube.com/watch?v=TQsP_PlCY1I&t=205s"
            )
            .video_id,
            Some("TQsP_PlCY1I".into())
        );
    }

    #[test]
    fn youtube_macro_takes_an_id_or_a_url_without_a_start() {
        let by_id = embed("youtube", "6cqVzgVQJfE");
        assert_eq!(by_id.video_id.as_deref(), Some("6cqVzgVQJfE"));
        let by_url = embed(
            "youtube",
            "https://www.youtube.com/watch?v=6cqVzgVQJfE&t=30",
        );
        assert_eq!(by_url.video_id.as_deref(), Some("6cqVzgVQJfE"));
        assert_eq!(by_url.start, None);
        assert_eq!(
            embed("vimeo", "76979871").embed_url,
            "https://player.vimeo.com/video/76979871"
        );
    }

    #[test]
    fn rejects_invalid_and_empty_arguments() {
        assert_eq!(video("video", &[]), Some(VideoArgument::Empty));
        assert_eq!(video("video", &[" ".into()]), Some(VideoArgument::Empty));
        assert_eq!(
            video("video", &["not a url".into()]),
            Some(VideoArgument::Invalid)
        );
        assert_eq!(
            video("video", &["example.com".into()]),
            Some(VideoArgument::Invalid)
        );
        assert_eq!(
            video("youtube", &["https://example.com/x".into()]),
            Some(VideoArgument::Invalid)
        );
        assert_eq!(video("query", &["x".into()]), None);
    }

    #[test]
    fn parses_and_displays_timestamps() {
        let show = |arg: &str| Timestamp::parse(arg).map(Timestamp::display);
        assert_eq!(show("0").as_deref(), Some("00:00"));
        assert_eq!(show("41").as_deref(), Some("00:41"));
        assert_eq!(show("620").as_deref(), Some("10:20"));
        assert_eq!(show("3600").as_deref(), Some("01:00:00"));
        assert_eq!(show("4:40").as_deref(), Some("04:40"));
        assert_eq!(show("1:02:03").as_deref(), Some("01:02:03"));
        assert_eq!(Timestamp::parse("4:40").unwrap().seconds, 280);
        assert_eq!(show("61:00"), None);
        assert_eq!(show("1:60"), None);
        assert_eq!(show("abc"), None);
        assert_eq!(show(""), None);
    }

    fn page(source: &str) -> PageMedia {
        let document = lsdoc::parse_to_projection(source);
        PageMedia::of(&document.blocks)
    }

    #[test]
    fn binds_timestamps_to_the_nearest_youtube_video_before_them() {
        let source = "- {{youtube-timestamp 5}} before any video\n\
- {{video https://www.youtube.com/watch?v=6cqVzgVQJfE}}\n\
\t- ## {{youtube-timestamp 280}} Chapter\n\
- {{video https://vimeo.com/76979871}}\n\
\t- {{youtube-timestamp 620}} after vimeo\n\
- {{video https://www.youtube.com/watch?v=TQsP_PlCY1I&t=205s}}\n\
- {{video https://www.youtube.com/watch?v=TQsP_PlCY1I}}\n\
\t- {{youtube-timestamp 10:20}} under the repeat\n";
        let media = page(source);
        assert_eq!(media.videos.len(), 4);
        let bound: Vec<_> = media
            .timestamp_at
            .values()
            .map(|(time, video)| (time.seconds, *video))
            .collect();
        assert_eq!(
            bound,
            vec![(5, None), (280, Some(0)), (620, Some(0)), (620, Some(3))]
        );
        let offset = source.find("{{video https://vimeo").unwrap();
        assert_eq!(media.video_at(offset).map(|(i, _)| i), Some(1));
        let (_, second) = media.timestamps().last().unwrap();
        assert_eq!(second.unwrap().start, None);
    }
}
