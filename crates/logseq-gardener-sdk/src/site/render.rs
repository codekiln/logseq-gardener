//! Escaped HTML rendering with bounded selected-content transclusion.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::ast::{Block, Inline, ListItem, Url};
use crate::garden::GardenDocument;

const MAX_DEPTH: usize = 64;

pub(super) struct Rendered {
    pub html: String,
    pub diagnostics: Vec<String>,
}

pub(super) fn escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        out.push_str(match c {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '"' => "&quot;",
            '\'' => "&#39;",
            _ => {
                out.push(c);
                continue;
            }
        });
    }
    out
}

/// A depth-limit violation withholds the document rather than assuming visibility.
pub(super) fn has_private_properties(blocks: &[Block]) -> bool {
    fn items_private(items: &[ListItem], depth: usize) -> bool {
        depth > MAX_DEPTH
            || items.iter().any(|item| {
                private(&item.content, depth + 1) || items_private(&item.items, depth + 1)
            })
    }
    fn private(blocks: &[Block], depth: usize) -> bool {
        depth > MAX_DEPTH
            || blocks.iter().any(|block| match block {
                Block::Properties { props, .. } => props.iter().any(|p| {
                    p.0.trim().eq_ignore_ascii_case("public")
                        && p.1.trim().eq_ignore_ascii_case("false")
                }),
                Block::Quote { children, .. } | Block::Custom { children, .. } => {
                    private(children, depth + 1)
                }
                Block::List { items, .. } => items_private(items, depth + 1),
                _ => false,
            })
    }
    private(blocks, 0)
}

/// Only published outlines supply labels and destinations.
#[derive(Clone)]
pub(super) struct BlockTarget {
    pub route: String,
    pub start: usize,
    pub label: String,
}

pub(super) fn uuid(value: &str) -> Option<String> {
    let value = value.trim();
    if value.len() != 36
        || !value.bytes().enumerate().all(|(i, c)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                c == b'-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
    {
        return None;
    }
    Some(value.to_ascii_lowercase())
}

pub(super) fn outline_targets(blocks: &[Block]) -> Vec<(String, usize, String)> {
    fn visit(blocks: &[Block], result: &mut Vec<(String, usize, String)>, depth: usize) {
        if depth > MAX_DEPTH {
            return;
        }
        for (i, block) in blocks.iter().enumerate() {
            if let Block::Bullet {
                inline,
                span: Some(span),
                ..
            } = block
                && let Some(Block::Properties { props, .. }) = blocks.get(i + 1)
            {
                let property = ["custom-id", "custom_id", "id"]
                    .into_iter()
                    .find_map(|key| {
                        props
                            .iter()
                            .rev()
                            .find(|p| p.0.trim().eq_ignore_ascii_case(key))
                    });
                if let Some(id) = property.and_then(|p| uuid(&p.1)) {
                    let label = block_label(inline, 0)
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ");
                    let mut label = label.chars().take(160).collect::<String>();
                    if label.chars().count() == 160 {
                        label.push('…');
                    }
                    result.push((id, span.0, label));
                }
            }
            match block {
                Block::Quote { children, .. } | Block::Custom { children, .. } => {
                    visit(children, result, depth + 1)
                }
                _ => {}
            }
        }
    }
    let mut result = Vec::new();
    visit(blocks, &mut result, 0);
    result
}

/// The parser stores outline bullets flat; include continuations and descendants.
fn outline_subtree(blocks: &[Block], start: usize, depth: usize) -> Option<&[Block]> {
    if depth > MAX_DEPTH {
        return None;
    }
    for (index, block) in blocks.iter().enumerate() {
        if let Block::Bullet {
            span: Some(span),
            level,
            ..
        } = block
            && span.0 == start
        {
            let end = blocks[index + 1..]
                .iter()
                .position(
                    |block| matches!(block, Block::Bullet { level: next, .. } if next <= level),
                )
                .map_or(blocks.len(), |offset| index + 1 + offset);
            return Some(&blocks[index..end]);
        }
        if let Block::Quote { children, .. } | Block::Custom { children, .. } = block
            && let Some(found) = outline_subtree(children, start, depth + 1)
        {
            return Some(found);
        }
    }
    None
}

/// Embedded outlines are flow content, so their ancestors cannot be phrasing-only.
fn contains_embed(nodes: &[Inline], depth: usize) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    nodes.iter().any(|node| match node {
        Inline::Macro { name, .. } => name.eq_ignore_ascii_case("embed"),
        Inline::Emphasis { children, .. }
        | Inline::Subscript { children, .. }
        | Inline::Superscript { children, .. }
        | Inline::Tag { children, .. } => contains_embed(children, depth + 1),
        _ => false,
    })
}

fn block_label(nodes: &[Inline], depth: usize) -> String {
    if depth > MAX_DEPTH {
        return "[nesting limit reached]".into();
    }
    let mut text = String::new();
    for node in nodes {
        match node {
            Inline::Plain { text: value, .. }
            | Inline::Code { text: value, .. }
            | Inline::Verbatim { text: value, .. } => text.push_str(value),
            Inline::Emphasis { children, .. }
            | Inline::Subscript { children, .. }
            | Inline::Superscript { children, .. }
            | Inline::Tag { children, .. } => text.push_str(&block_label(children, depth + 1)),
            Inline::Link { url, label, .. } => {
                if !label.is_empty() {
                    text.push_str(&block_label(label, depth + 1));
                } else if let Url::PageRef { v } = url {
                    text.push_str(v);
                } else {
                    text.push_str("[reference]");
                }
            }
            Inline::Macro { name, .. } if name.eq_ignore_ascii_case("embed") => {
                text.push_str("[embedded content]");
            }
            _ => text.push_str("[unsupported content]"),
        }
    }
    text
}

pub(super) fn render_document_with_embeds(
    document: &GardenDocument,
    pages: &crate::page_names::PageNameIndex<String>,
    targets: &BTreeMap<String, Option<BlockTarget>>,
    route: &str,
    documents: &BTreeMap<String, &GardenDocument>,
    asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
) -> Rendered {
    let anchors = outline_targets(&document.parsed.blocks)
        .into_iter()
        .filter_map(|(id, start, _)| {
            targets
                .get(&id)
                .and_then(Option::as_ref)
                .filter(|target| target.route == route && target.start == start)
                .map(|_| (start, id))
        })
        .collect();
    let mut renderer = Renderer {
        html: String::new(),
        diagnostics: Vec::new(),
        pages,
        targets,
        anchors,
        documents,
        source: document.relative_path.clone(),
        active: vec![route.to_owned()],
        remaining_expansions: 128,
        remaining_work: 10_000,
        embedded: false,
        asset,
    };
    renderer.blocks(&document.parsed.blocks, 0);
    Rendered {
        html: renderer.html,
        diagnostics: renderer.diagnostics,
    }
}

struct Renderer<'a> {
    html: String,
    diagnostics: Vec<String>,
    pages: &'a crate::page_names::PageNameIndex<String>,
    targets: &'a BTreeMap<String, Option<BlockTarget>>,
    anchors: BTreeMap<usize, String>,
    documents: &'a BTreeMap<String, &'a GardenDocument>,
    source: PathBuf,
    active: Vec<String>,
    remaining_expansions: usize,
    remaining_work: usize,
    embedded: bool,
    asset: &'a mut dyn FnMut(&Path, &str) -> Result<String, String>,
}

impl Renderer<'_> {
    fn diagnostic(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.diagnostics.push(if self.embedded {
            format!("{}: {message}", self.source.display())
        } else {
            message
        });
    }
    fn text(&mut self, text: &str) {
        self.html.push_str(&escape(text));
    }
    fn literal(&mut self, kind: &str, text: &str) {
        self.diagnostic(format!("Unsupported {kind}; preserved as literal text"));
        self.html.push_str("<code>");
        self.text(text);
        self.html.push_str("</code>");
    }
    fn consume_work(&mut self) -> bool {
        if !self.embedded {
            return true;
        }
        if self.remaining_work == 0 {
            self.literal(
                "embed macro total-work budget",
                "[transclusion work limit reached]",
            );
            return false;
        }
        self.remaining_work -= 1;
        true
    }
    fn blocks(&mut self, blocks: &[Block], depth: usize) {
        if depth > MAX_DEPTH {
            self.literal("deeply nested content", "[nesting limit reached]");
            return;
        }
        // The AST's outline bullets are flat. Track observed levels, without creating
        // empty list items for skipped indentation levels. Continuations remain in li.
        let mut levels: Vec<(u32, bool)> = Vec::new();
        for (index, block) in blocks.iter().enumerate() {
            if !self.consume_work() {
                break;
            }
            if let Block::Bullet {
                level,
                size,
                inline,
                marker,
                priority,
                htags,
                span,
                ..
            } = block
            {
                let collapsed = matches!(blocks.get(index + 1), Some(Block::Properties { props, .. })
                    if props.iter().rev().find(|p| p.0.trim().eq_ignore_ascii_case("collapsed"))
                        .is_some_and(|p| p.1.trim().eq_ignore_ascii_case("true")))
                    && blocks[index + 1..]
                        .iter()
                        .find_map(|block| match block {
                            Block::Bullet { level, .. } => Some(level),
                            _ => None,
                        })
                        .is_some_and(|next| next > level);
                let item_start = if collapsed {
                    "<li class=\"outline-collapsed\">"
                } else {
                    "<li>"
                };
                while levels.last().is_some_and(|(previous, _)| previous > level) {
                    self.close_outline_item(levels.pop().unwrap().1);
                    self.html.push_str("</ul>");
                }
                if levels.last().is_some_and(|(previous, _)| previous == level) {
                    self.close_outline_item(levels.last().unwrap().1);
                    self.html.push_str(item_start);
                } else if levels.len() < MAX_DEPTH {
                    self.html.push_str("<ul>");
                    self.html.push_str(item_start);
                    levels.push((*level, false));
                } else {
                    self.diagnostic("Outline nesting limit reached; rendered as sibling");
                    self.close_outline_item(levels.last().unwrap().1);
                    self.html.push_str(item_start);
                }
                levels.last_mut().unwrap().1 = collapsed;
                if !self.embedded
                    && let Some(id) = span.as_ref().and_then(|span| self.anchors.get(&span.0))
                {
                    self.html
                        .push_str(&format!("<span id=\"block-{id}\"></span>"));
                }
                let flow_label = collapsed && contains_embed(inline, 0);
                if collapsed {
                    self.html
                        .push_str("<details class=\"outline-disclosure\"><summary>");
                    if flow_label {
                        self.text(&block_label(inline, 0));
                    }
                }
                if !flow_label {
                    self.heading_content(
                        *size,
                        inline,
                        marker.as_deref(),
                        priority.as_deref(),
                        htags,
                        depth,
                    );
                }
                if collapsed {
                    self.html.push_str("</summary>");
                }
                if flow_label {
                    self.heading_content(
                        *size,
                        inline,
                        marker.as_deref(),
                        priority.as_deref(),
                        htags,
                        depth,
                    );
                }
                continue;
            }
            match block {
                Block::Properties { .. } | Block::Comment { .. } | Block::CommentBlock { .. } => {}
                Block::Paragraph { inline, .. } => {
                    let tag = if contains_embed(inline, 0) {
                        "div"
                    } else {
                        "p"
                    };
                    self.html.push_str(&format!("<{tag}>"));
                    self.inlines(inline, depth + 1, false);
                    self.html.push_str(&format!("</{tag}>"));
                }
                Block::Heading {
                    level,
                    size,
                    inline,
                    marker,
                    priority,
                    htags,
                    ..
                } => {
                    self.heading_content(
                        Some(size.unwrap_or(*level)),
                        inline,
                        marker.as_deref(),
                        priority.as_deref(),
                        htags,
                        depth,
                    );
                }
                Block::List { items, .. } => self.list(items, depth + 1),
                Block::Quote { children, .. } => {
                    self.html.push_str("<blockquote>");
                    self.blocks(children, depth + 1);
                    self.html.push_str("</blockquote>");
                }
                Block::Custom { name, children, .. } => {
                    self.diagnostic(format!(
                        "Unsupported custom block {name}; rendered children"
                    ));
                    self.html.push_str("<aside>");
                    self.text(name);
                    self.blocks(children, depth + 1);
                    self.html.push_str("</aside>");
                }
                Block::Src { code, .. } | Block::Example { code, .. } => {
                    self.html.push_str("<pre><code>");
                    self.text(code);
                    self.html.push_str("</code></pre>");
                }
                Block::Table { header, rows, .. } => {
                    self.html.push_str("<table>");
                    if let Some(header) = header {
                        self.row(header, "th", depth);
                    }
                    for row in rows {
                        self.row(row, "td", depth);
                    }
                    self.html.push_str("</table>");
                }
                Block::Hr { .. } => self.html.push_str("<hr>"),
                Block::FootnoteDef { name, inline, .. } => {
                    self.diagnostic("Footnote definition rendered without reference navigation");
                    self.html.push_str("<p>");
                    self.text(&format!("[{name}] "));
                    self.inlines(inline, depth + 1, false);
                    self.html.push_str("</p>");
                }
                Block::RawHtml { text, .. } => self.literal("HTML", text),
                Block::Hiccup { v, .. } => self.literal("Hiccup", v),
                Block::DisplayedMath { text, .. } => self.literal("math", text),
                Block::Export { name, content, .. } => {
                    self.literal("export block", &format!("{name}: {content}"))
                }
                Block::LatexEnv { name, content, .. } => {
                    self.literal("LaTeX environment", &format!("{name}: {content}"))
                }
                Block::Drawer { name, .. } => self.literal("drawer", name),
                Block::Directive { name, value, .. } => {
                    self.literal("directive", &format!("{name}: {value}"))
                }
                Block::Results { .. } => self.literal("results", "[results unsupported]"),
                Block::Bullet { .. } => unreachable!("handled above"),
            }
        }
        for (_, collapsed) in levels.into_iter().rev() {
            self.close_outline_item(collapsed);
            self.html.push_str("</ul>");
        }
    }
    fn close_outline_item(&mut self, collapsed: bool) {
        if collapsed {
            self.html.push_str("</details>");
        }
        self.html.push_str("</li>");
    }
    fn heading_content(
        &mut self,
        size: Option<u32>,
        inline: &[Inline],
        marker: Option<&str>,
        priority: Option<&str>,
        tags: &[String],
        depth: usize,
    ) {
        let size = size
            .filter(|_| !contains_embed(inline, 0))
            .map(|size| size.clamp(1, 6));
        if let Some(size) = size {
            self.html.push_str(&format!("<h{size}>"));
        }
        for prefix in [marker, priority].into_iter().flatten() {
            self.text(prefix);
            self.html.push(' ');
        }
        self.inlines(inline, depth + 1, false);
        for tag in tags {
            self.html.push(' ');
            self.text(&format!("#{tag}"));
        }
        if let Some(size) = size {
            self.html.push_str(&format!("</h{size}>"));
        }
    }
    fn list(&mut self, items: &[ListItem], depth: usize) {
        if depth > MAX_DEPTH {
            self.literal("deeply nested list", "[nesting limit reached]");
            return;
        }
        let mut previous = None;
        for item in items {
            if !self.consume_work() {
                break;
            }
            let tag = if item.ordered { "ol" } else { "ul" };
            if previous != Some(tag) {
                if let Some(previous) = previous {
                    self.html.push_str(&format!("</{previous}>"));
                }
                self.html.push_str(&format!("<{tag}>"));
                previous = Some(tag);
            }
            self.html.push_str("<li>");
            if let Some(checked) = item.checkbox {
                self.text(if checked { "[x] " } else { "[ ] " });
            }
            self.inlines(&item.name, depth + 1, false);
            self.blocks(&item.content, depth + 1);
            self.list(&item.items, depth + 1);
            self.html.push_str("</li>");
        }
        if let Some(previous) = previous {
            self.html.push_str(&format!("</{previous}>"));
        }
    }
    fn row(&mut self, row: &[Vec<Inline>], tag: &str, depth: usize) {
        self.html.push_str("<tr>");
        for cell in row {
            self.html.push_str(&format!("<{tag}>"));
            self.inlines(cell, depth + 1, false);
            self.html.push_str(&format!("</{tag}>"));
        }
        self.html.push_str("</tr>");
    }
    fn inlines(&mut self, nodes: &[Inline], depth: usize, label: bool) {
        if depth > MAX_DEPTH {
            self.literal("deeply nested inline", "[nesting limit reached]");
            return;
        }
        for node in nodes {
            if !self.consume_work() {
                break;
            }
            match node {
                Inline::Plain { text, .. } | Inline::Target { text, .. } => self.text(text),
                Inline::Code { text, .. } | Inline::Verbatim { text, .. } => {
                    self.html.push_str("<code>");
                    self.text(text);
                    self.html.push_str("</code>");
                }
                Inline::Break { .. } | Inline::HardBreak { .. } => self.html.push_str("<br>"),
                Inline::Emphasis { emph, children, .. } => {
                    if !label && contains_embed(children, 0) {
                        self.inlines(children, depth + 1, label);
                        continue;
                    }
                    let tag = match emph.as_str() {
                        "Bold" => "strong",
                        "Italic" => "em",
                        "Strike_through" => "del",
                        "Highlight" => "mark",
                        "Underline" => "u",
                        _ => {
                            self.diagnostic(format!("Unsupported emphasis {emph}"));
                            "span"
                        }
                    };
                    self.html.push_str(&format!("<{tag}>"));
                    self.inlines(children, depth + 1, label);
                    self.html.push_str(&format!("</{tag}>"));
                }
                Inline::Subscript { children, .. } | Inline::Superscript { children, .. } => {
                    if !label && contains_embed(children, 0) {
                        self.inlines(children, depth + 1, label);
                        continue;
                    }
                    let tag = if matches!(node, Inline::Subscript { .. }) {
                        "sub"
                    } else {
                        "sup"
                    };
                    self.html.push_str(&format!("<{tag}>"));
                    self.inlines(children, depth + 1, label);
                    self.html.push_str(&format!("</{tag}>"));
                }
                Inline::Link {
                    url,
                    label: text,
                    image,
                    full,
                    ..
                } => {
                    if label {
                        // No nested anchors, and no asset requests hidden inside link labels.
                        self.diagnostic("Nested link or image rendered as literal label");
                        self.text(full);
                    } else {
                        self.link(url, text, *image, depth + 1);
                    }
                }
                Inline::NestedLink { content, .. } => {
                    if label {
                        self.text(content);
                    } else {
                        self.page_link(content, &[], depth + 1);
                    }
                }
                Inline::Tag { children, .. } => {
                    self.html.push('#');
                    self.inlines(children, depth + 1, label);
                }
                Inline::Macro { name, args, .. }
                    if name.eq_ignore_ascii_case("embed") && !label =>
                {
                    self.embed(args, depth + 1);
                }
                Inline::Macro { name, args, .. } => self.literal(
                    "macro (including embeds and queries)",
                    &format!("{{{{{name} {}}}}}", args.join(", ")),
                ),
                Inline::ExportSnippet { name, content, .. } => {
                    self.literal("export snippet", &format!("@@{name}: {content}@@"))
                }
                Inline::Latex { mode, body, .. } => {
                    self.literal("math", &format!("{mode}: {body}"))
                }
                Inline::Timestamp { ts, date, .. } => {
                    self.literal("timestamp", &format!("{ts}: {date}"))
                }
                Inline::Cookie { value, total, .. } => self.text(&total.map_or_else(
                    || format!("[{value}%]"),
                    |total| format!("[{value}/{total}]"),
                )),
                Inline::Fnref {
                    name, definition, ..
                } => {
                    self.diagnostic("Footnote reference rendered without navigation");
                    self.text(&format!("[{name}]"));
                    self.inlines(definition, depth + 1, label);
                }
                Inline::InlineHtml { text, .. } => self.literal("inline HTML", text),
                Inline::Email { text, .. } => {
                    if let (Some(local), Some(domain)) = (
                        text.get("local_part").and_then(|value| value.as_str()),
                        text.get("domain").and_then(|value| value.as_str()),
                    ) {
                        let address = format!("{local}@{domain}");
                        if local.is_empty()
                            || domain.is_empty()
                            || address.chars().any(|c| c.is_control() || c.is_whitespace())
                        {
                            self.literal("email syntax", &address);
                        } else if label {
                            self.text(&address);
                        } else {
                            // Keep address delimiters out of mailto query/header syntax.
                            let destination = address
                                .replace('%', "%25")
                                .replace('?', "%3F")
                                .replace('#', "%23");
                            self.html
                                .push_str(&format!("<a href=\"mailto:{}\">", escape(&destination)));
                            self.text(&address);
                            self.html.push_str("</a>");
                        }
                    } else {
                        self.literal("email syntax", "[unsupported email]");
                    }
                }
                Inline::Entity { unicode, .. } => self.text(unicode),
                Inline::Hiccup { v, .. } => self.literal("inline Hiccup", v),
            }
        }
    }
    fn embed(&mut self, args: &[String], depth: usize) {
        let argument = args.join(", ");
        let argument = argument.trim();
        let fallback = format!("{{{{embed {argument}}}}}");
        let resolved = if let Some(page) = argument
            .strip_prefix("[[")
            .and_then(|v| v.strip_suffix("]]"))
        {
            match self.pages.candidates(page) {
                [route] => Some((route.clone(), None)),
                _ => None,
            }
        } else if let Some(id) = argument
            .strip_prefix("((")
            .and_then(|v| v.strip_suffix("))"))
            .and_then(uuid)
        {
            self.targets
                .get(&id)
                .and_then(Option::as_ref)
                .map(|target| (target.route.clone(), Some(target.start)))
        } else {
            None
        };
        let Some((route, start)) = resolved else {
            self.literal("embed macro target (unavailable or ambiguous)", &fallback);
            return;
        };
        let Some(document) = self.documents.get(&route).copied() else {
            self.literal("embed macro target (unavailable)", &fallback);
            return;
        };
        let identity = start.map_or_else(|| route.clone(), |start| format!("{route}#{start}"));
        if self.active.contains(&identity) || self.active.len() >= 16 || depth > MAX_DEPTH {
            self.literal("embed macro cycle or depth limit", &fallback);
            return;
        }
        if self.remaining_expansions == 0 || self.remaining_work == 0 {
            self.literal("embed macro expansion budget", &fallback);
            return;
        }
        let blocks = match start {
            Some(start) => outline_subtree(&document.parsed.blocks, start, 0),
            None => Some(document.parsed.blocks.as_slice()),
        };
        let Some(blocks) = blocks else {
            self.literal("embed macro outline target (unavailable)", &fallback);
            return;
        };
        self.remaining_expansions -= 1;
        self.active.push(identity);
        let previous_source = std::mem::replace(&mut self.source, document.relative_path.clone());
        let previous_embedded = std::mem::replace(&mut self.embedded, true);
        self.html.push_str("<section class=\"transclusion\">");
        self.blocks(blocks, depth + 1);
        self.html.push_str("</section>");
        self.source = previous_source;
        self.embedded = previous_embedded;
        self.active.pop();
    }
    fn label(&mut self, label: &[Inline], fallback: &str, depth: usize) {
        if label.is_empty() {
            self.text(fallback);
        } else {
            self.inlines(label, depth, true);
        }
    }
    fn page_link(&mut self, target: &str, label: &[Inline], depth: usize) {
        let candidates = self.pages.candidates(target);
        if let [route] = candidates {
            self.html
                .push_str(&format!("<a href=\"{}\">", escape(route)));
            self.label(label, target, depth);
            self.html.push_str("</a>");
        } else {
            let reason = if candidates.len() > 1 {
                "Ambiguous selected page reference"
            } else {
                "Unresolved or excluded page reference"
            };
            self.diagnostic(format!("{reason}: {target}"));
            self.html.push_str("<span class=\"unresolved\">");
            self.label(label, target, depth);
            self.html.push_str("</span>");
        }
    }
    fn media(&mut self, route: &str, label: &[Inline], fallback: &str, depth: usize) -> bool {
        let path = route.split(['?', '#']).next().unwrap_or(route);
        let path = if let Some((_, remainder)) = path.split_once("://") {
            remainder.split_once('/').map_or("", |(_, path)| path)
        } else {
            path
        };
        let audio = is_audio(path);
        if !audio && !is_raster_image(path) {
            return false;
        }
        let name = if label.is_empty() {
            fallback.to_owned()
        } else {
            let (text, unsupported) = plain_label(label, depth);
            if unsupported {
                self.diagnostic("Unsupported media label syntax; rendered placeholder");
            }
            text
        };
        let name = escape(&name);
        let remote = is_https_media_url(route);
        let route = escape(route);
        if audio {
            self.html.push_str(&format!(
                "<audio controls preload=\"none\" aria-label=\"{name}\" src=\"{route}\">{name}</audio> "
            ));
            if remote {
                self.html
                    .push_str(&format!("<a href=\"{route}\">Open recording: {name}</a>"));
            } else {
                self.html
                    .push_str(&format!("<a href=\"{route}\" download>{name}</a>"));
            }
        } else {
            self.html
                .push_str(&format!("<img src=\"{route}\" alt=\"{name}\">"));
        }
        true
    }

    fn link(&mut self, url: &Url, label: &[Inline], image: bool, depth: usize) {
        if let Url::PageRef { v } = url {
            if image {
                self.diagnostic("Page image reference unsupported");
                self.label(label, v, depth);
            } else {
                self.page_link(v, label, depth);
            }
            return;
        }
        if let Url::BlockRef { v } = url {
            if !image
                && let Some(target) = uuid(v).and_then(|id| {
                    self.targets
                        .get(&id)
                        .and_then(Option::as_ref)
                        .map(|target| (id, target))
                })
            {
                let (id, target) = target;
                self.html.push_str(&format!(
                    "<a href=\"{}#block-{id}\">",
                    escape(&target.route)
                ));
                let fallback = if target.label.is_empty() {
                    format!("(({v}))")
                } else {
                    target.label.clone()
                };
                self.label(label, &fallback, depth);
                self.html.push_str("</a>");
            } else {
                self.diagnostic(format!("Unavailable or ambiguous block reference: {v}"));
                self.label(label, &format!("(({v}))"), depth);
            }
            return;
        }
        let destination = match url {
            Url::File { v } | Url::Search { v } | Url::EmbedData { v } => v.clone(),
            Url::Complex { protocol, link } => match (protocol, link) {
                (Some(protocol), Some(link)) => format!("{protocol}://{link}"),
                (_, link) => link.clone().unwrap_or_default(),
            },
            Url::PageRef { .. } | Url::BlockRef { .. } => unreachable!(),
        };
        let lower = destination.to_ascii_lowercase();
        let scheme = lower.split_once(':').map(|(scheme, _)| scheme);
        let allowed = matches!(scheme, Some("http" | "https" | "mailto"))
            && !destination.chars().any(char::is_control);
        if image && is_https_media_url(&destination) {
            if !self.media(&destination, label, &destination, depth) {
                self.diagnostic(format!("Unsupported remote media: {destination}"));
                self.label(label, &destination, depth);
            }
        } else if allowed && !image {
            self.html
                .push_str(&format!("<a href=\"{}\">", escape(&destination)));
            self.label(label, &destination, depth);
            self.html.push_str("</a>");
        } else if scheme.is_some() || destination.starts_with("//") {
            self.diagnostic(format!("Unsupported URL or remote image: {destination}"));
            self.label(label, &destination, depth);
        } else {
            match (self.asset)(&self.source, &destination) {
                Ok(route) => {
                    if image && self.media(&route, label, &destination, depth) {
                        return;
                    }
                    if image {
                        self.diagnostic(format!("Non-image asset requested with image syntax; rendered as link: {destination}"));
                    }
                    self.html
                        .push_str(&format!("<a href=\"{}\">", escape(&route)));
                    self.label(label, &destination, depth);
                    self.html.push_str("</a>");
                }
                Err(reason) => {
                    self.diagnostic(format!("Unsupported local asset {destination}: {reason}"));
                    self.label(label, &destination, depth);
                }
            }
        }
    }
}

fn is_https_media_url(destination: &str) -> bool {
    if destination
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
    {
        return false;
    }
    let Some((scheme, remainder)) = destination.split_once("://") else {
        return false;
    };
    let authority = remainder.split(['/', '?', '#']).next().unwrap_or_default();
    scheme.eq_ignore_ascii_case("https") && !authority.is_empty() && !authority.contains('@')
}

fn is_audio(path: &str) -> bool {
    path.rsplit_once('.').is_some_and(|(_, extension)| {
        matches!(
            extension.to_ascii_lowercase().as_str(),
            "mp3" | "wav" | "ogg"
        )
    })
}

fn is_raster_image(route: &str) -> bool {
    route.rsplit_once('.').is_some_and(|(_, extension)| {
        matches!(
            extension.to_ascii_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif" | "ico"
        )
    })
}

fn plain_label(nodes: &[Inline], depth: usize) -> (String, bool) {
    if depth > MAX_DEPTH {
        return ("[nesting limit reached]".into(), true);
    }
    let mut text = String::new();
    let mut unsupported = false;
    for node in nodes {
        match node {
            Inline::Plain { text: value, .. }
            | Inline::Code { text: value, .. }
            | Inline::Verbatim { text: value, .. } => text.push_str(value),
            Inline::Emphasis { children, .. }
            | Inline::Subscript { children, .. }
            | Inline::Superscript { children, .. }
            | Inline::Tag { children, .. } => {
                let (child_text, child_unsupported) = plain_label(children, depth + 1);
                text.push_str(&child_text);
                unsupported |= child_unsupported;
            }
            _ => {
                text.push_str("[unsupported image label]");
                unsupported = true;
            }
        }
    }
    (text, unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::DocumentKind;

    fn render_document(
        document: &GardenDocument,
        pages: &BTreeMap<String, String>,
        asset: &mut dyn FnMut(&str) -> Result<String, String>,
    ) -> Rendered {
        let mut names = crate::page_names::PageNameIndex::default();
        for (title, route) in pages {
            let mut without_aliases = document.clone();
            without_aliases.parsed.blocks.clear();
            names.insert(route.clone(), title, &without_aliases);
        }
        super::render_document_with_embeds(
            document,
            &names,
            &BTreeMap::new(),
            "p-test.html",
            &BTreeMap::new(),
            &mut |_, url| asset(url),
        )
    }
    fn document(source: &str) -> GardenDocument {
        GardenDocument {
            relative_path: "pages/Notes___Start.md".into(),
            kind: DocumentKind::Page,
            source: source.into(),
            parsed: lsdoc::parse_to_projection(source),
        }
    }
    fn render(source: &str) -> Rendered {
        render_document(
            &document(source),
            &BTreeMap::from([("notes/other".into(), "p-other.html".into())]),
            &mut |_| Err("not an asset".into()),
        )
    }

    #[test]
    fn outlines_preserve_hierarchy_and_supported_content() {
        let rendered = render(
            "- Parent **bold**\n  - Child\n    - Grandchild\n  - Sibling\n- ## Heading\n  ```rust\n  let x = \"<tag>\";\n  ```\n",
        );
        assert!(rendered.html.contains("<ul><li>Parent <strong>bold</strong><ul><li>Child<ul><li>Grandchild</li></ul></li><li>Sibling"), "{}", rendered.html);
        assert!(rendered.html.contains("<h2>Heading</h2>"));
        assert!(rendered.html.contains("&lt;tag&gt;"));
        assert_eq!(
            rendered.html.matches("<ul>").count(),
            rendered.html.matches("</ul>").count()
        );
        assert_eq!(
            rendered.html.matches("<li>").count(),
            rendered.html.matches("</li>").count()
        );
    }

    #[test]
    fn selected_links_resolve_without_transclusion() {
        let rendered = render(
            "- [[Notes/Other]] [[Notes/Private/Hidden]] ((12345678-1234-1234-1234-123456789abc)) {{embed [[Notes/Private/Hidden]]}}\n",
        );
        assert!(rendered.html.contains("href=\"p-other.html\""));
        assert!(rendered.html.contains("Notes/Private/Hidden"));
        assert_eq!(rendered.html.matches("<a ").count(), 1);
        assert!(
            rendered
                .diagnostics
                .iter()
                .any(|d| d.contains("Unresolved or excluded"))
        );
        assert!(
            rendered
                .diagnostics
                .iter()
                .any(|d| d.contains("block reference"))
        );
        assert!(rendered.diagnostics.iter().any(|d| d.contains("macro")));
    }

    #[test]
    fn html_urls_and_remote_images_are_safe_literals() {
        let rendered = render(
            "- <script>alert(1)</script> [bad](javascript:alert) [ok](https://example.com/?x=1&y=2) ![remote](https://example.com/image.svg) [:script \"evil\"]\n",
        );
        assert!(!rendered.html.contains("<script"));
        assert!(!rendered.html.contains("href=\"javascript:"));
        assert!(!rendered.html.contains("<img"));
        assert!(rendered.html.contains("&lt;script&gt;"));
        assert!(
            rendered
                .html
                .contains("href=\"https://example.com/?x=1&amp;y=2\"")
        );
        assert!(rendered.diagnostics.iter().any(|d| d.contains("HTML")));
        assert!(rendered.diagnostics.iter().any(|d| d.contains("Hiccup")));
    }

    #[test]
    fn only_rendered_assets_request_copies() {
        let mut requested = Vec::new();
        let rendered = render_document(
            &document(
                "- ![photo](../assets/photo.png) {{embed [[Other]]}} ![remote](http://example.com/remote.png)\n",
            ),
            &BTreeMap::new(),
            &mut |url| {
                requested.push(url.to_string());
                Ok("assets/a.png".into())
            },
        );
        assert_eq!(requested, ["../assets/photo.png"]);
        assert!(
            rendered
                .html
                .contains("<img src=\"assets/a.png\" alt=\"photo\">")
        );
    }

    #[test]
    fn audio_assets_in_image_syntax_have_controls_and_other_files_keep_links() {
        let mut requested = Vec::new();
        let rendered = render_document(
            &document(
                "- ![Paper](../assets/paper.pdf) ![Song](../assets/song.mp3) ![Film](../assets/film.mp4)\n",
            ),
            &BTreeMap::new(),
            &mut |url| {
                requested.push(url.to_string());
                Ok(format!("assets/copied.{}", url.rsplit('.').next().unwrap()))
            },
        );
        assert_eq!(
            requested,
            [
                "../assets/paper.pdf",
                "../assets/song.mp3",
                "../assets/film.mp4"
            ]
        );
        assert!(!rendered.html.contains("<img"));
        assert!(
            rendered
                .html
                .contains("<a href=\"assets/copied.pdf\">Paper</a>")
        );
        assert!(
            rendered
                .html
                .contains("<audio controls preload=\"none\" aria-label=\"Song\" src=\"assets/copied.mp3\">Song</audio> <a href=\"assets/copied.mp3\" download>Song</a>")
        );
        assert!(
            rendered
                .html
                .contains("<a href=\"assets/copied.mp4\">Film</a>")
        );
        assert_eq!(
            rendered
                .diagnostics
                .iter()
                .filter(|d| d.contains("Non-image asset"))
                .count(),
            2
        );
    }

    #[test]
    fn private_properties_are_recursive_and_code_is_literal() {
        assert!(has_private_properties(
            &document("- Parent\n  - Child\n    public:: false\n")
                .parsed
                .blocks
        ));
        assert!(!has_private_properties(
            &document("- ```\n  public:: false\n  ```\n").parsed.blocks
        ));
        assert!(!has_private_properties(
            &document("public:: true\n- text\n").parsed.blocks
        ));
        let property = document("public:: false\n").parsed.blocks;
        let quote = Block::Quote {
            children: property,
            span: None,
        };
        assert!(has_private_properties(&[quote]));
    }

    #[test]
    fn nesting_limit_is_safe_and_private_check_fails_closed() {
        let mut blocks = document("- leaf\n").parsed.blocks;
        for _ in 0..70 {
            blocks = vec![Block::Quote {
                children: blocks,
                span: None,
            }];
        }
        assert!(has_private_properties(&blocks));
        let mut doc = document("");
        doc.parsed.blocks = blocks;
        let rendered = render_document(&doc, &BTreeMap::new(), &mut |_| panic!("unexpected asset"));
        assert!(
            rendered
                .diagnostics
                .iter()
                .any(|d| d.contains("deeply nested"))
        );
    }

    #[test]
    fn https_media_has_plain_labels_and_never_requests_local_assets() {
        let rendered = render_document(
            &document(
                "- ![Song & \"live\"](https://media.example/song.MP3?x=1&y=2#start) ![Artwork](https://media.example/art.gif) [Download](https://media.example/song.mp3)\n",
            ),
            &BTreeMap::new(),
            &mut |_| panic!("remote media must not request local assets"),
        );
        assert!(rendered.html.contains(
            "<audio controls preload=\"none\" aria-label=\"Song &amp; &quot;live&quot;\""
        ));
        assert!(
            rendered
                .html
                .contains("src=\"https://media.example/song.MP3?x=1&amp;y=2#start\"")
        );
        assert!(rendered.html.contains(
            "<a href=\"https://media.example/song.MP3?x=1&amp;y=2#start\">Open recording: Song &amp; &quot;live&quot;</a>"
        ));
        assert!(!rendered.html.contains(" download>"));
        let uppercase = render("- ![Song](HTTPS://media.example/song.mp3)\n");
        assert!(
            uppercase
                .html
                .contains("<a href=\"HTTPS://media.example/song.mp3\">Open recording: Song</a>")
        );
        assert!(!uppercase.html.contains(" download>"));
        assert!(
            rendered
                .html
                .contains("<img src=\"https://media.example/art.gif\" alt=\"Artwork\">")
        );
        assert!(
            rendered
                .html
                .contains("<a href=\"https://media.example/song.mp3\">Download</a>")
        );
        assert!(
            rendered.diagnostics.is_empty(),
            "{:?}",
            rendered.diagnostics
        );
    }

    #[test]
    fn malformed_and_unsupported_remote_media_keeps_labels() {
        for url in [
            "http://media.example/song.mp3",
            "javascript:evil.mp3",
            "https:///song.mp3",
            "https://user@media.example/song.mp3",
            "https://media.example/song.svg",
            "https://song.mp3",
            "https://media.example/so\\ng.mp3",
            "https://media.example/song.mp3\u{7f}",
        ] {
            let rendered = render_document(
                &document(&format!("- ![Recording]({url})\n")),
                &BTreeMap::new(),
                &mut |_| Err("unsupported".into()),
            );
            assert!(
                !rendered.html.contains("<audio"),
                "{url}: {}",
                rendered.html
            );
            assert!(!rendered.html.contains("<img"), "{url}: {}", rendered.html);
            assert!(rendered.html.contains("Recording"), "{url}");
            assert!(!rendered.diagnostics.is_empty(), "{url}");
        }
    }

    #[test]
    fn email_autolinks_preserve_addresses_and_literal_code() {
        let rendered = render(
            "- Contact <reader+news@example.org>.\n- Literal `<reader@example.org>`.\n- ~~~text\n  <reader@example.org>\n  ~~~\n",
        );
        assert!(
            rendered
                .html
                .contains("<a href=\"mailto:reader+news@example.org\">reader+news@example.org</a>")
        );
        assert_eq!(rendered.html.matches("mailto:").count(), 1);
        assert!(
            rendered
                .html
                .contains("<code>&lt;reader@example.org&gt;</code>")
        );
        assert!(rendered.html.contains("&lt;reader@example.org&gt;"));
        assert!(
            rendered.diagnostics.is_empty(),
            "{:?}",
            rendered.diagnostics
        );
    }

    fn email_document(local: &str, domain: &str) -> GardenDocument {
        let mut doc = document("<reader@example.org>");
        let Block::Paragraph { inline, .. } = &mut doc.parsed.blocks[0] else {
            panic!("expected email paragraph");
        };
        let Inline::Email { text, .. } = &mut inline[0] else {
            panic!("expected email node");
        };
        text["local_part"] = local.into();
        text["domain"] = domain.into();
        doc
    }

    #[test]
    fn email_addresses_escape_html_and_mailto_delimiters() {
        let rendered = render_document(
            &email_document("o'hara&news?subject=hello#tag%20\"", "example.org"),
            &BTreeMap::new(),
            &mut |_| panic!("email must not request an asset"),
        );
        assert!(
            rendered.html.contains(
                "href=\"mailto:o&#39;hara&amp;news%3Fsubject=hello%23tag%2520&quot;@example.org\""
            ),
            "{}",
            rendered.html
        );
        assert!(
            rendered
                .html
                .contains(">o&#39;hara&amp;news?subject=hello#tag%20&quot;@example.org</a>")
        );
        assert!(rendered.diagnostics.is_empty());
    }

    #[test]
    fn emails_in_link_labels_remain_text() {
        let mut doc = email_document("reader", "example.org");
        let Block::Paragraph { inline, .. } = &mut doc.parsed.blocks[0] else {
            unreachable!();
        };
        let email = inline[0].clone();
        inline[0] = Inline::Link {
            url: Url::Complex {
                protocol: Some("https".into()),
                link: Some("example.org".into()),
            },
            label: vec![email],
            full: String::new(),
            image: false,
            metadata: String::new(),
            title: None,
            span: None,
        };
        let rendered =
            render_document(&doc, &BTreeMap::new(), &mut |_| panic!("email label asset"));
        assert_eq!(rendered.html.matches("<a ").count(), 1);
        assert!(
            rendered
                .html
                .contains("<a href=\"https://example.org\">reader@example.org</a>")
        );
        assert!(rendered.diagnostics.is_empty());
    }

    #[test]
    fn malformed_email_nodes_keep_visible_fallbacks() {
        for (local, domain) in [
            ("", "example.org"),
            ("reader", ""),
            ("reader\n", "example.org"),
            ("reader", "bad domain"),
        ] {
            let rendered = render_document(
                &email_document(local, domain),
                &BTreeMap::new(),
                &mut |_| panic!("malformed email asset"),
            );
            assert!(!rendered.html.contains("<a "));
            assert!(rendered.html.contains("<code>"));
            assert_eq!(rendered.diagnostics.len(), 1);
        }
        let mut doc = email_document("reader", "example.org");
        let Block::Paragraph { inline, .. } = &mut doc.parsed.blocks[0] else {
            unreachable!();
        };
        let Inline::Email { text, .. } = &mut inline[0] else {
            unreachable!();
        };
        text["domain"] = false.into();
        let rendered = render_document(&doc, &BTreeMap::new(), &mut |_| panic!("email asset"));
        assert!(rendered.html.contains("[unsupported email]"));
        assert!(!rendered.html.contains("<a "));
        assert_eq!(rendered.diagnostics.len(), 1);
        let mut doc = email_document("reader", "example.org");
        let Block::Paragraph { inline, .. } = &mut doc.parsed.blocks[0] else {
            unreachable!();
        };
        let Inline::Email { text, .. } = &mut inline[0] else {
            unreachable!();
        };
        text.as_object_mut().unwrap().remove("domain");
        let rendered = render_document(&doc, &BTreeMap::new(), &mut |_| panic!("email asset"));
        assert!(rendered.html.contains("[unsupported email]"));
        assert!(!rendered.html.contains("<a "));
        assert_eq!(rendered.diagnostics.len(), 1);
    }

    #[test]
    fn nested_labels_cannot_create_anchors_or_copy_assets() {
        let mut doc = document("");
        let image = Inline::Link {
            url: Url::Search {
                v: "../assets/hidden.png".into(),
            },
            label: vec![],
            full: "![hidden](../assets/hidden.png)".into(),
            image: true,
            metadata: String::new(),
            title: None,
            span: None,
        };
        doc.parsed.blocks = vec![Block::Paragraph {
            inline: vec![Inline::Link {
                url: Url::PageRef {
                    v: "Notes/Other".into(),
                },
                label: vec![image],
                full: String::new(),
                image: false,
                metadata: String::new(),
                title: None,
                span: None,
            }],
            span: None,
        }];
        let rendered = render_document(
            &doc,
            &BTreeMap::from([("notes/other".into(), "other.html".into())]),
            &mut |_| panic!("label asset must not be copied"),
        );
        assert_eq!(rendered.html.matches("<a ").count(), 1);
        assert!(!rendered.html.contains("<img"));
        assert!(
            rendered
                .diagnostics
                .iter()
                .any(|d| d.contains("Nested link"))
        );
    }
}
