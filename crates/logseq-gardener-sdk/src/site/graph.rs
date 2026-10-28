//! The graph view (`graph.html`), its data files and the Page graph's neighbourhood files.
//!
//! The SDK's [`GraphIndex`] answers which nodes and edges each graph holds; this module lays
//! them out, encodes them and writes the page that draws them.
use std::collections::BTreeSet;
use std::io::Write;
use std::path::PathBuf;
use std::thread::JoinHandle;

use flate2::Compression;
use flate2::write::GzEncoder;

use super::graph_layout::cached_layout;
use super::render::escape;
use crate::graph::{ForceSettings, Graph, GraphIndex, GraphSettings, GraphSource};

/// The graph view page.
pub(super) const PAGE_ROUTE: &str = "graph.html";
/// Data files and neighbourhood files live here.
pub(super) const DIRECTORY: &str = "_graph";
pub(super) const DEFAULT_DATA: &str = "_graph/default.bin";
pub(super) const JOURNALS_DATA: &str = "_graph/journals.bin";
/// Neighbourhood files: `_graph/p/<route hash>.json`.
pub(super) const NEIGHBOURHOODS: &str = "_graph/p";
pub(super) const SCRIPT_ROUTE: &str = "lsg-graph.js";
pub(super) const WORKER_ROUTE: &str = "lsg-graph-worker.js";
pub(super) const STYLES_ROUTE: &str = "lsg-graph.css";
/// The data file format; the script refuses other versions.
pub(super) const FORMAT_VERSION: u16 = 1;
/// The largest neighbourhood file, gzip; larger page graphs read the global file.
pub(super) const NEIGHBOURHOOD_LIMIT: usize = 100_000;

/// A script without indentation or whole-line comments, which keeps it within its budget.
fn compact(source: &str) -> String {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn script() -> String {
    compact(include_str!("graph.js"))
}

pub(super) fn worker() -> String {
    compact(include_str!("graph-worker.js"))
}

pub(super) fn styles() -> String {
    compact(include_str!("graph.css"))
}

fn varint(out: &mut Vec<u8>, mut value: u32) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

/// Encode a graph and its positions in the data file format, before [`gzip`]:
///
/// - `LSGG`, the format version (u16) and a reserved u16;
/// - node count, edge count, `N links` count and the titles' byte length (u32 each);
/// - the positions' bounds: x min, y min, x max, y max (f32 each);
/// - each title's byte length (varint), then the titles' UTF-8 bytes;
/// - one flags byte per node, then each node's degree (varint);
/// - each node's x and y as u16 between the bounds;
/// - edges sorted by first then second node, each as the first node's increase (varint) and
///   the second node's distance past the previous second node, or past the first node when
///   the first changed, less one (varint).
///
/// Numbers are little-endian. A node's page is `p-<FNV-1a 64 of its title>.html` unless
/// its flags mark it reference-only.
pub(super) fn encode<K>(graph: &Graph<K>, positions: &[(f32, f32)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"LSGG");
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes());
    let titles: usize = graph.nodes.iter().map(|node| node.title.len()).sum();
    for count in [graph.nodes.len(), graph.edges.len(), graph.links, titles] {
        out.extend_from_slice(&(count as u32).to_le_bytes());
    }
    let (mut x0, mut y0, mut x1, mut y1) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
    if let Some(&(x, y)) = positions.first() {
        (x0, y0, x1, y1) = (x, y, x, y);
    }
    for &(x, y) in positions {
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x);
        y1 = y1.max(y);
    }
    for bound in [x0, y0, x1, y1] {
        out.extend_from_slice(&bound.to_le_bytes());
    }
    for node in &graph.nodes {
        varint(&mut out, node.title.len() as u32);
    }
    for node in &graph.nodes {
        out.extend_from_slice(node.title.as_bytes());
    }
    out.extend(graph.nodes.iter().map(|node| node.flags));
    for node in &graph.nodes {
        varint(&mut out, node.degree);
    }
    let scale = |value: f32, low: f32, high: f32| {
        if high > low {
            (((value - low) / (high - low)) * 65535.0).round() as u16
        } else {
            0
        }
    };
    for &(x, y) in positions {
        out.extend_from_slice(&scale(x, x0, x1).to_le_bytes());
        out.extend_from_slice(&scale(y, y0, y1).to_le_bytes());
    }
    let (mut previous_a, mut previous_b) = (0, 0);
    for &(a, b) in &graph.edges {
        let base = if a == previous_a { previous_b } else { a };
        varint(&mut out, a - previous_a);
        varint(&mut out, b - base - 1);
        previous_a = a;
        previous_b = b;
    }
    out
}

/// A page's neighbourhood as the Page graph card reads it: `t` titles (the page first),
/// `f` flags and `e` edges as flat node pairs.
pub(super) fn neighbourhood_json<K>(graph: &Graph<K>) -> String {
    let titles: Vec<&str> = graph.nodes.iter().map(|node| node.title.as_str()).collect();
    let flags: Vec<u8> = graph.nodes.iter().map(|node| node.flags).collect();
    let edges: Vec<u32> = graph.edges.iter().flat_map(|&(a, b)| [a, b]).collect();
    serde_json::json!({"v": FORMAT_VERSION, "t": titles, "f": flags, "e": edges}).to_string()
}

/// A data file is stored gzip-compressed, so that it transfers at its compressed size from
/// any host; the script decompresses it with `DecompressionStream`.
pub(super) fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(bytes).expect("in-memory gzip");
    encoder.finish().expect("in-memory gzip")
}

pub(super) fn gzip_len(bytes: &[u8]) -> usize {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes).expect("in-memory gzip");
    encoder.finish().expect("in-memory gzip").len()
}

/// Graph files being computed on other threads while pages render: the two layouts and the
/// neighbourhood files.
pub(super) struct Pending {
    layouts: Vec<(&'static str, JoinHandle<Vec<u8>>)>,
    neighbourhoods: JoinHandle<Vec<(String, String)>>,
    /// Whether a journals graph is written.
    pub journals: bool,
}

impl Pending {
    /// Read the garden graph and start laying it out and writing neighbourhoods.
    pub fn start(
        source: &GraphSource<'_, String>,
        journals: bool,
        forces: ForceSettings,
        cache: Option<PathBuf>,
    ) -> Self {
        let index = GraphIndex::new(source);
        let settings = if journals {
            vec![false, true]
        } else {
            vec![false]
        };
        let layouts = settings
            .into_iter()
            .map(|with_journals| {
                let graph = index.graph(GraphSettings::everything(with_journals));
                let cache = cache.clone();
                let route = if with_journals {
                    JOURNALS_DATA
                } else {
                    DEFAULT_DATA
                };
                let worker = std::thread::spawn(move || {
                    let titles: Vec<&str> = graph.nodes.iter().map(|n| n.title.as_str()).collect();
                    let (positions, _) =
                        cached_layout(cache.as_deref(), &titles, &graph.edges, forces);
                    gzip(&encode(&graph, &positions))
                });
                (route, worker)
            })
            .collect();
        let neighbourhoods = std::thread::spawn(move || {
            let mut files = Vec::new();
            for (route, graph) in index.neighbourhoods() {
                let graph = if journals {
                    graph
                } else {
                    graph.without_journal_only()
                };
                let json = neighbourhood_json(&graph);
                if let Some(hash) = route
                    .strip_prefix("p-")
                    .and_then(|r| r.strip_suffix(".html"))
                    && gzip_len(json.as_bytes()) <= NEIGHBOURHOOD_LIMIT
                {
                    files.push((format!("{NEIGHBOURHOODS}/{hash}.json"), json));
                }
            }
            files
        });
        Self {
            layouts,
            neighbourhoods,
            journals,
        }
    }

    /// The neighbourhood files, once they are ready.
    pub fn neighbourhoods(&mut self) -> Vec<(String, String)> {
        let worker = std::mem::replace(&mut self.neighbourhoods, std::thread::spawn(Vec::new));
        worker.join().expect("graph neighbourhoods panicked")
    }

    /// Wait for the layouts and return the data files.
    pub fn finish(self) -> Vec<(&'static str, Vec<u8>)> {
        self.layouts
            .into_iter()
            .map(|(route, worker)| (route, worker.join().expect("graph layout panicked")))
            .collect()
    }
}

/// Names with no published page that the graph must still leave out: every garden page
/// that is not published.
pub(super) fn hidden_names(
    all: &BTreeSet<String>,
    published: &BTreeSet<String>,
) -> BTreeSet<String> {
    all.difference(published).cloned().collect()
}

/// The main column of `graph.html`: the canvas, the settings panel and a message for readers
/// without JavaScript.
pub(super) fn main(
    all_pages: &str,
    settings: GraphSettings,
    forces: ForceSettings,
    journals_file: bool,
) -> String {
    let switch = |name: &str, label: &str, on: bool| {
        format!(
            "<div class=\"graph-filters-row\"><span id=\"lsg-graph-{name}-label\">{label}</span><button type=\"button\" role=\"switch\" class=\"lsg-switch\" data-setting=\"{name}\" aria-checked=\"{on}\" aria-labelledby=\"lsg-graph-{name}-label\"></button></div>"
        )
    };
    let slider = |name: &str, label: &str, min: i32, max: i32, step: i32, value: i32| {
        format!(
            "<label class=\"graph-filters-slider\"><span>{label} <output data-for=\"{name}\">{value}</output></span><input type=\"range\" data-force=\"{name}\" min=\"{min}\" max=\"{max}\" step=\"{step}\" value=\"{value}\"></label>"
        )
    };
    let section = |name: &str, title: &str, body: String| {
        format!(
            "<section class=\"graph-filters-section\" data-section=\"{name}\"><button type=\"button\" class=\"graph-filters-head\" aria-expanded=\"false\" aria-controls=\"lsg-graph-{name}\"><span>{title}</span><svg class=\"lsg-icon\" viewBox=\"0 0 24 24\" aria-hidden=\"true\"><path d=\"M6 9l6 6l6 -6\"/></svg></button><div class=\"graph-filters-body\" id=\"lsg-graph-{name}\" hidden>{body}</div></section>"
        )
    };
    let journals_switch = if journals_file {
        switch("journals", "Journals", settings.journals)
    } else {
        String::new()
    };
    let nodes = format!(
        "<p class=\"graph-filters-count\" data-count=\"pages\"></p>{journals_switch}{}{}{}<label class=\"graph-filters-slider\" data-hops hidden><span>N hops from selected nodes <output data-for=\"hops\">1</output></span><input type=\"range\" data-hops-input min=\"1\" max=\"10\" step=\"1\" value=\"1\"></label><button type=\"button\" class=\"graph-filters-link\" data-reset=\"graph\">Reset Graph</button>",
        switch("orphan_pages", "Orphan pages", settings.orphan_pages),
        switch("builtin_pages", "Built-in pages", settings.builtin_pages),
        switch("excluded_pages", "Excluded pages", settings.excluded_pages),
    );
    let search = "<button type=\"button\" class=\"graph-filters-link\" data-search-open>Click to search</button><input type=\"search\" class=\"graph-filters-input\" data-search-input aria-label=\"Search graph nodes\" placeholder=\"Filter by title\" hidden><ul class=\"graph-filters-chips\" data-filters></ul><button type=\"button\" class=\"graph-filters-link\" data-clear hidden>Clear All</button><ul class=\"graph-filters-results\" data-results aria-label=\"Matching nodes\"></ul>".to_owned();
    let force_rows = format!(
        "<p class=\"graph-filters-count\" data-count=\"links\"></p>{}{}{}{}<button type=\"button\" class=\"graph-filters-link\" data-reset=\"forces\">Reset Forces</button>",
        switch("paused", "Pause simulation", false),
        slider(
            "link_distance",
            "Link Distance",
            10,
            180,
            10,
            forces.link_distance
        ),
        slider(
            "charge_strength",
            "Charge Strength",
            -1000,
            1000,
            100,
            forces.charge_strength
        ),
        slider(
            "charge_range",
            "Charge Range",
            500,
            4000,
            100,
            forces.charge_range
        ),
    );
    let export = "<button type=\"button\" class=\"graph-filters-link\" data-export>as PNG</button>"
        .to_owned();
    let config = serde_json::json!({
        "settings": {
            "journals": settings.journals,
            "orphan_pages": settings.orphan_pages,
            "builtin_pages": settings.builtin_pages,
            "excluded_pages": settings.excluded_pages,
        },
        "forces": {
            "link_distance": forces.link_distance,
            "charge_strength": forces.charge_strength,
            "charge_range": forces.charge_range,
        },
        "data": DEFAULT_DATA,
        "journals": journals_file.then_some(JOURNALS_DATA),
    });
    format!(
        "<h1 class=\"lsg-graph-title\">Graph</h1><div class=\"lsg-graph\" id=\"global-graph\" data-lsg-graph=\"{config}\"><canvas class=\"lsg-graph-canvas\" tabindex=\"0\" aria-label=\"Graph of the garden's pages. Drag or use the arrow keys to pan, scroll or press + and - to zoom.\"></canvas><p class=\"lsg-graph-status\" role=\"status\" aria-live=\"polite\"></p><noscript><p class=\"lsg-graph-noscript\">The graph view needs JavaScript. <a href=\"{all}\">All pages</a> lists every page.</p></noscript><div class=\"graph-filters\" hidden>{}{}{}{}</div></div>",
        section("nodes", "Nodes", nodes),
        section("search", "Search", search),
        section("forces", "Forces", force_rows),
        section("export", "Export", export),
        config = escape(&config.to_string()),
        all = escape(all_pages),
    )
}

/// `graph.html`'s head: the styles and the script, deferred.
pub(super) fn head() -> String {
    format!(
        "<link rel=\"stylesheet\" href=\"{STYLES_ROUTE}\"><script defer src=\"{SCRIPT_ROUTE}\"></script>"
    )
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::graph::GraphNode;

    /// A decoded data file: titles, flags, degrees, positions, edges and links.
    pub(crate) type Decoded = (
        Vec<String>,
        Vec<u8>,
        Vec<u32>,
        Vec<(u16, u16)>,
        Vec<(u32, u32)>,
        u32,
    );

    /// Decode a data file.
    pub(crate) fn decode(bytes: &[u8]) -> Decoded {
        assert_eq!(&bytes[..4], b"LSGG");
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), FORMAT_VERSION);
        let word = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let (n, e, links, text) = (
            word(8) as usize,
            word(12) as usize,
            word(16),
            word(20) as usize,
        );
        let mut at = 40;
        let read = |at: &mut usize| {
            let (mut value, mut shift) = (0_u32, 0);
            loop {
                let byte = bytes[*at];
                *at += 1;
                value |= u32::from(byte & 0x7f) << shift;
                if byte < 0x80 {
                    return value;
                }
                shift += 7;
            }
        };
        let lengths: Vec<usize> = (0..n).map(|_| read(&mut at) as usize).collect();
        let mut titles = Vec::new();
        for length in lengths {
            titles.push(String::from_utf8(bytes[at..at + length].to_vec()).unwrap());
            at += length;
        }
        assert_eq!(titles.iter().map(String::len).sum::<usize>(), text);
        let flags = bytes[at..at + n].to_vec();
        at += n;
        let degrees = (0..n).map(|_| read(&mut at)).collect();
        let positions = (0..n)
            .map(|i| {
                let p = at + i * 4;
                (
                    u16::from_le_bytes([bytes[p], bytes[p + 1]]),
                    u16::from_le_bytes([bytes[p + 2], bytes[p + 3]]),
                )
            })
            .collect();
        at += n * 4;
        let (mut a, mut b) = (0, 0);
        let mut edges = Vec::new();
        for _ in 0..e {
            let da = read(&mut at);
            let base = if da == 0 { b } else { a + da };
            a += da;
            b = base + read(&mut at) + 1;
            edges.push((a, b));
        }
        assert_eq!(at, bytes.len());
        (titles, flags, degrees, positions, edges, links)
    }

    fn sample() -> Graph<String> {
        let node = |title: &str, flags| GraphNode {
            title: title.to_owned(),
            page: None,
            degree: 300,
            flags,
        };
        Graph {
            nodes: vec![
                node("tmux", 2),
                node("Ünïcode/名前", 64),
                node("c", 0),
                node("d", 1),
            ],
            edges: vec![(0, 1), (0, 3), (1, 2), (2, 3)],
            links: 9,
        }
    }

    #[test]
    fn data_files_round_trip() {
        let graph = sample();
        let positions = [(-100.0, 5.0), (100.0, -5.0), (0.0, 0.0), (50.0, 5.0)];
        let bytes = encode(&graph, &positions);
        let (titles, flags, degrees, quantised, edges, links) = decode(&bytes);
        assert_eq!(titles, ["tmux", "Ünïcode/名前", "c", "d"]);
        assert_eq!(flags, [2, 64, 0, 1]);
        assert_eq!(degrees, [300; 4]);
        assert_eq!(quantised[0], (0, 65535));
        assert_eq!(quantised[1], (65535, 0));
        assert_eq!(edges, graph.edges);
        assert_eq!(links, 9);
    }

    /// tests/graph.test.cjs reads this file with the browser's reader.
    #[test]
    fn javascript_fixture_matches_the_encoder() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/graph-sample.bin");
        let positions = [(-100.0, 5.0), (100.0, -5.0), (0.0, 0.0), (50.0, 5.0)];
        let bytes = gzip(&encode(&sample(), &positions));
        if std::env::var_os("LSG_UPDATE_FIXTURES").is_some() {
            std::fs::write(&path, &bytes).unwrap();
        }
        let stored = std::fs::read(&path).unwrap();
        let mut plain = Vec::new();
        std::io::Read::read_to_end(
            &mut flate2::read::GzDecoder::new(stored.as_slice()),
            &mut plain,
        )
        .unwrap();
        assert_eq!(
            plain,
            encode(&sample(), &positions),
            "regenerate with LSG_UPDATE_FIXTURES=1 cargo test -p logseq-gardener-sdk javascript_fixture"
        );
    }

    #[test]
    fn neighbourhood_json_lists_titles_flags_and_edges() {
        let json = neighbourhood_json(&sample());
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["t"][0], "tmux");
        assert_eq!(value["f"][1], 64);
        assert_eq!(value["e"].as_array().unwrap().len(), 8);
    }

    #[test]
    fn graph_page_holds_the_panel_and_a_message_without_javascript() {
        let html = main(
            "pages.html",
            GraphSettings::default(),
            ForceSettings::default(),
            true,
        );
        for text in [
            "Nodes",
            "Search",
            "Forces",
            "Export",
            "Journals",
            "Orphan pages",
            "Built-in pages",
            "Excluded pages",
            "Reset Graph",
            "Click to search",
            "Pause simulation",
            "Link Distance",
            "Charge Strength",
            "Charge Range",
            "Reset Forces",
            "as PNG",
            "class=\"graph-filters\"",
            "needs JavaScript",
            "href=\"pages.html\"",
        ] {
            assert!(html.contains(text), "{text}");
        }
        let order: Vec<usize> = ["\"nodes\"", "\"search\"", "\"forces\"", "\"export\""]
            .iter()
            .map(|name| html.find(&format!("data-section={name}")).unwrap())
            .collect();
        assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(html.matches("aria-expanded=\"false\"").count(), 4);
        assert!(
            !main(
                "index.html",
                GraphSettings::default(),
                ForceSettings::default(),
                false
            )
            .contains("Journals")
        );
    }
}
