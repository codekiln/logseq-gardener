//! Build-time highlighting for fenced code, in the markup Logseq's CodeMirror theme styles.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex};

use syntect::easy::ScopeRangeIterator;
use syntect::parsing::{ParseState, Scope, ScopeStack, SyntaxReference, SyntaxSet};

use crate::ast::Block;
use crate::code_blocks::{LogseqMode, code_language, code_text};
use crate::garden::GardenDocument;

pub(super) const STYLES: &str = include_str!("code.css");

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(two_face::syntax::extra_newlines);

/// TextMate scope prefixes and the CodeMirror token class each one shows as. The
/// innermost scope with a match wins; the first matching prefix in this list wins.
const SCOPE_CLASSES: &[(&str, &str)] = &[
    ("comment", "cm-comment"),
    ("string.regexp", "cm-string-2"),
    ("constant.character.escape", "cm-string-2"),
    ("string", "cm-string"),
    ("constant.numeric", "cm-number"),
    ("constant", "cm-atom"),
    ("keyword.operator", "cm-operator"),
    ("keyword", "cm-keyword"),
    ("storage", "cm-keyword"),
    ("variable.language", "cm-keyword"),
    ("entity.name.tag", "cm-tag"),
    ("entity.other.attribute-name", "cm-attribute"),
    ("entity.other.inherited-class", "cm-type"),
    ("entity.name", "cm-def"),
    ("support.type.property-name", "cm-property"),
    ("support.type", "cm-type"),
    ("support.class", "cm-type"),
    ("support.function", "cm-builtin"),
    ("support.macro", "cm-builtin"),
    ("variable.function", "cm-variable"),
    ("variable", "cm-variable"),
    ("markup.heading", "cm-header"),
    ("markup.quote", "cm-quote"),
    ("markup.underline.link", "cm-link"),
    ("meta.preprocessor", "cm-meta"),
    ("meta.annotation", "cm-meta"),
];

/// Scopes a mode's CodeMirror tokenizer leaves plain although another mode colours them:
/// CodeMirror's shell mode shows command names as plain text.
const PLAIN_SCOPES: &[(&str, &str)] = &[("shell", "variable.function")];

/// Modes whose CodeMirror tokenizer marks every other identifier as a variable.
const IDENTIFIER_VARIABLE_MODES: &[&str] = &["rust", "javascript", "python", "clike", "go", "lua"];

/// The grammar for a mode, where it differs from CodeMirror's name for the language.
fn syntax(mode: &LogseqMode) -> Option<&'static SyntaxReference> {
    let name = match mode.name {
        "Shell" => "Bourne Again Shell (bash)",
        "Properties files" => "INI",
        "Common Lisp" => "Lisp",
        "Fortran" => "Fortran (Modern)",
        "Protobuf" => "Protocol Buffer",
        "reStructuredText" => "reStructuredText",
        "TypeScript-JSX" => "TypeScriptReact",
        "JSON-LD" => "JSON",
        "Vue.js Component" => "Vue Component",
        "Objective-C" => "Objective-C",
        "SCSS" => "SCSS",
        "HTTP" => "HTTP Request and Response",
        "SystemVerilog" => "SystemVerilog",
        "LaTeX" | "sTeX" => "LaTeX",
        other => other,
    };
    SYNTAXES
        .find_syntax_by_name(name)
        .or_else(|| SYNTAXES.find_syntax_by_token(mode.mode))
        .filter(|syntax| syntax.name != "Plain Text")
}

type Highlighted = Option<Vec<String>>;
type Classes = HashMap<(&'static str, Scope), Option<&'static str>>;

/// Blocks already highlighted, by MIME type and code. A block shown again in linked
/// references or an embed reuses its markup.
static HIGHLIGHTED: LazyLock<Mutex<HashMap<(&'static str, String), Highlighted>>> =
    LazyLock::new(Mutex::default);

thread_local! {
    static CLASSES: RefCell<Classes> = RefCell::default();
}

/// Code lines as escaped HTML with CodeMirror token spans, one string per line. `None`
/// when the build has no grammar for the mode or the grammar fails, so the block shows
/// plain text.
pub(super) fn highlight(code: &str, mode: &LogseqMode) -> Highlighted {
    let key = (mode.mime, code.to_owned());
    if let Some(lines) = cache().get(&key) {
        return lines.clone();
    }
    let lines = CLASSES.with_borrow_mut(|classes| tokenize(code, mode, classes));
    cache().insert(key, lines.clone());
    lines
}

fn cache() -> std::sync::MutexGuard<'static, HashMap<(&'static str, String), Highlighted>> {
    HIGHLIGHTED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Highlight every distinct fenced block of the documents across the machine's cores,
/// so rendering pages one at a time finds each block's markup ready.
pub(super) fn prepare<'a>(documents: impl IntoIterator<Item = &'a GardenDocument>) {
    let mut seen = HashSet::new();
    let mut jobs = Vec::new();
    for document in documents {
        collect(
            &document.source,
            &document.parsed.blocks,
            &mut seen,
            &mut jobs,
        );
    }
    // Longest blocks first, taken one at a time, so no thread is left with the slowest ones.
    jobs.sort_by_key(|(_, code)| std::cmp::Reverse(code.len()));
    let next = AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(1, usize::from);
    let results: Vec<_> = std::thread::scope(|scope| {
        (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut classes = HashMap::new();
                    let mut done = Vec::new();
                    while let Some((mode, code)) = jobs.get(next.fetch_add(1, Ordering::Relaxed)) {
                        done.push((
                            (mode.mime, code.clone()),
                            tokenize(code, mode, &mut classes),
                        ));
                    }
                    done
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|handle| handle.join().unwrap_or_default())
            .collect()
    });
    cache().extend(results);
}

fn collect_items(
    source: &str,
    items: &[crate::ast::ListItem],
    seen: &mut HashSet<(&'static str, String)>,
    jobs: &mut Vec<(LogseqMode, String)>,
) {
    for item in items {
        collect(source, &item.content, seen, jobs);
        collect_items(source, &item.items, seen, jobs);
    }
}

fn collect(
    source: &str,
    blocks: &[Block],
    seen: &mut HashSet<(&'static str, String)>,
    jobs: &mut Vec<(LogseqMode, String)>,
) {
    for block in blocks {
        match block {
            Block::Src { lang, .. } => {
                let (Some(mode), Some(code)) = (code_language(lang).mode, code_text(source, block))
                else {
                    continue;
                };
                if seen.insert((mode.mime, code.clone())) {
                    jobs.push((mode, code));
                }
            }
            Block::Quote { children, .. } | Block::Custom { children, .. } => {
                collect(source, children, seen, jobs)
            }
            Block::List { items, .. } => collect_items(source, items, seen, jobs),
            _ => {}
        }
    }
}

fn tokenize(code: &str, mode: &LogseqMode, classes: &mut Classes) -> Option<Vec<String>> {
    let syntax = syntax(mode)?;
    let variables =
        IDENTIFIER_VARIABLE_MODES.contains(&mode.mode) && mode.mime != "application/json";
    let mut state = ParseState::new(syntax);
    let mut stack = ScopeStack::new();
    let mut input = String::new();
    let mut lines = Vec::new();
    for line in code.split('\n') {
        input.clear();
        input.push_str(line);
        input.push('\n');
        let ops = state.parse_line(&input, &SYNTAXES).ok()?;
        let mut tokens = Tokens::default();
        for (range, op) in ScopeRangeIterator::new(&ops, &input) {
            stack.apply(op).ok()?;
            if range.start >= line.len() || range.is_empty() {
                continue;
            }
            let text = &input[range.start..range.end.min(line.len())];
            let class = stack.as_slice().iter().rev().find_map(|scope| {
                *classes
                    .entry((mode.mode, *scope))
                    .or_insert_with(|| class_for(mode.mode, &scope.build_string()))
            });
            match class {
                None if variables => tokens.identifiers(text),
                class => tokens.push(text, class),
            }
        }
        lines.push(tokens.finish());
    }
    Some(lines)
}

fn class_for(mode: &str, scope: &str) -> Option<&'static str> {
    let matches = |prefix: &str| {
        scope == prefix
            || scope
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('.'))
    };
    if PLAIN_SCOPES
        .iter()
        .any(|(plain_mode, prefix)| *plain_mode == mode && matches(prefix))
    {
        return None;
    }
    SCOPE_CLASSES
        .iter()
        .find(|(prefix, _)| matches(prefix))
        .map(|(_, class)| *class)
}

/// One line's tokens; adjacent text with the same class shares one span.
#[derive(Default)]
struct Tokens {
    html: String,
    run: String,
    class: Option<&'static str>,
}

impl Tokens {
    fn push(&mut self, text: &str, class: Option<&'static str>) {
        let blank = text.trim().is_empty();
        if class != self.class && !(blank && !self.run.is_empty()) {
            self.flush();
            self.class = class;
        }
        self.run.push_str(text);
    }
    fn identifiers(&mut self, text: &str) {
        let mut rest = text;
        while let Some(start) = rest.find(|c: char| c.is_alphabetic() || c == '_') {
            self.push(&rest[..start], None);
            let len = rest[start..]
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .unwrap_or(rest.len() - start);
            self.push(&rest[start..start + len], Some("cm-variable"));
            rest = &rest[start + len..];
        }
        self.push(rest, None);
    }
    fn flush(&mut self) {
        if self.run.is_empty() {
            return;
        }
        let text = super::render::escape(&self.run);
        match self.class {
            Some(class) => self
                .html
                .push_str(&format!("<span class=\"{class}\">{text}</span>")),
            None => self.html.push_str(&text),
        }
        self.run.clear();
    }
    fn finish(mut self) -> String {
        self.flush();
        self.html
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_blocks::code_language;

    fn html(fence: &str, code: &str) -> String {
        highlight(code, &code_language(fence).mode.unwrap())
            .unwrap()
            .join("\n")
    }

    fn has(html: &str, class: &str, text: &str) -> bool {
        html.contains(&format!("<span class=\"{class}\">{text}</span>"))
    }

    #[test]
    fn rust_keywords_strings_and_numbers() {
        let out = html(
            "rust",
            "fn main() {\n    let mut n = 0;\n    'outer: loop { break 'outer; }\n    println!(\"hi\");\n}",
        );
        let keywords: Vec<&str> = out
            .split("<span class=\"cm-keyword\">")
            .skip(1)
            .flat_map(|span| span.split("</span>").next().unwrap().split_whitespace())
            .collect();
        for keyword in ["fn", "let", "mut", "loop", "break"] {
            assert!(keywords.contains(&keyword), "{keyword} in {out}");
        }
        assert!(has(&out, "cm-string", "&quot;hi&quot;"), "{out}");
        assert!(has(&out, "cm-number", "0"), "{out}");
        assert!(has(&out, "cm-def", "main"), "{out}");
        assert_eq!(out.lines().count(), 5);
    }

    #[test]
    fn every_garden_language_gets_tokens() {
        for (fence, code, class) in [
            ("bash", "# note\necho \"hi\" $HOME", "cm-comment"),
            ("sh", "if true; then echo 1; fi", "cm-keyword"),
            ("zsh", "export A=\"b\"", "cm-string"),
            ("json", "{\"a\": 1}", "cm-number"),
            ("toml", "[a]\nb = \"c\"", "cm-string"),
            ("python", "def f():\n    return 'x'", "cm-keyword"),
            ("yaml", "a: 'b'", "cm-string"),
            ("yml", "a: 1", "cm-number"),
            ("typescript", "const a: number = 1;", "cm-keyword"),
            ("ts", "let s = 'x';", "cm-string"),
            ("markdown", "# Title", "cm-header"),
            ("lua", "local x = 1 -- c", "cm-comment"),
            ("javascript", "function f() { return 2 }", "cm-keyword"),
            ("dockerfile", "FROM rust:1\nRUN echo hi", "cm-keyword"),
            ("clojure", "(defn f [x] \"s\")", "cm-string"),
            ("edn", "{:a 1}", "cm-number"),
            ("xml", "<a href=\"b\"/>", "cm-tag"),
            ("css", "a { color: red; }", "cm-tag"),
            ("c", "int main() { return 0; }", "cm-number"),
            ("cpp", "int x = 1;", "cm-number"),
            ("go", "func main() {}", "cm-keyword"),
            ("haskell", "main = putStrLn \"hi\"", "cm-string"),
            ("ini", "[a]\nb=1 ; c", "cm-comment"),
            ("java", "class A { int x = 1; }", "cm-keyword"),
        ] {
            let out = html(fence, code);
            assert!(out.contains(&format!("\"{class}\"")), "{fence}: {out}");
        }
    }

    #[test]
    fn markup_is_escaped_and_lines_kept() {
        let out = html("bash", "echo '<b>&'\n\necho 2");
        assert!(out.contains("&lt;b&gt;&amp;"), "{out}");
        assert_eq!(out.split('\n').count(), 3);
    }

    #[test]
    fn scope_prefixes_match_whole_segments() {
        assert_eq!(
            class_for("rust", "keyword.operator.rust"),
            Some("cm-operator")
        );
        assert_eq!(class_for("rust", "keywords.rust"), None);
        assert_eq!(class_for("rust", "comment"), Some("cm-comment"));
    }

    #[test]
    fn shell_command_names_stay_plain() {
        let out = html("bash", "rust-script hello.rs");
        assert!(!out.contains("cm-"), "{out}");
    }
}
