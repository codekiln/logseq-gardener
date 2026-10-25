//! Fenced code as Logseq's published site shows it: the text without the outline's
//! indentation, and the CodeMirror mode the fence language selects.

use crate::ast::Block;

/// A code block's text without the indentation that places it under its bullet.
///
/// For a fenced block the prefix is the fence line's indentation, or the bullet's
/// indentation plus two spaces when the fence opens on the bullet line. A line that
/// starts with less of the prefix loses only the part it has, and whitespace inside the
/// fence beyond the prefix stays. Example blocks arrive from the parser without the
/// prefix. One trailing newline is dropped. Other blocks give `None`.
pub fn code_text(source: &str, block: &Block) -> Option<String> {
    let (code, prefix) = match block {
        Block::Src { code, span, .. } => (
            code.as_str(),
            span.as_ref()
                .map_or("".into(), |span| fence_prefix(source, span.0)),
        ),
        Block::Example { code, .. } => (code.as_str(), String::new()),
        _ => return None,
    };
    let code = code.strip_suffix('\n').unwrap_or(code);
    Some(
        code.split('\n')
            .map(|line| strip_prefix(line, &prefix))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn fence_prefix(source: &str, start: usize) -> String {
    let Some(line_start) = source
        .get(..start)
        .map(|s| s.rfind('\n').map_or(0, |i| i + 1))
    else {
        return String::new();
    };
    let line = &source[line_start..];
    let indent = &line[..line.len() - line.trim_start_matches([' ', '\t']).len()];
    if line[indent.len()..].starts_with("- ") {
        format!("{indent}  ")
    } else {
        indent.to_owned()
    }
}

fn strip_prefix<'a>(line: &'a str, prefix: &str) -> &'a str {
    let shared = line
        .bytes()
        .zip(prefix.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    &line[shared..]
}

/// A fence language as Logseq's published site reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeLanguage {
    /// The fence language in lower case, shown as the block's label; `None` without one.
    pub label: Option<String>,
    /// The CodeMirror mode Logseq highlights the block with; `None` shows plain text.
    pub mode: Option<LogseqMode>,
}

/// One of the CodeMirror modes Logseq's published site bundles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogseqMode {
    /// CodeMirror's name for the language, such as `Shell`.
    pub name: &'static str,
    /// The CodeMirror mode that tokenizes it, such as `shell`.
    pub mode: &'static str,
    /// The MIME type that selects it, such as `text/x-sh`.
    pub mime: &'static str,
}

/// Look up a fence language the way Logseq's published site does.
///
/// `edn`, `clj`, `cljc`, `cljs` and `clojurescript` become `clojure`; the lower-cased
/// language is then compared with each mode's name and aliases
/// (`CodeMirror.findModeByName`), and failing that with the bundled mode names, which
/// CodeMirror also accepts. Anything else, such as `py`, `text` or `mermaid`, has no mode.
pub fn code_language(fence: &str) -> CodeLanguage {
    let label = fence.trim().to_lowercase();
    if label.is_empty() {
        return CodeLanguage {
            label: None,
            mode: None,
        };
    }
    let name = match label.as_str() {
        "edn" | "clj" | "cljc" | "cljs" | "clojurescript" => "clojure",
        other => other,
    };
    let entry = MODES
        .iter()
        .find(|(mode_name, aliases, ..)| {
            mode_name.to_lowercase() == name || aliases.contains(&name)
        })
        .or_else(|| MODES.iter().find(|(_, _, mode, _)| *mode == name));
    CodeLanguage {
        mode: entry.map(|&(name, _, mode, mime)| LogseqMode { name, mode, mime }),
        label: Some(label),
    }
}

/// `CodeMirror.modeInfo` from Logseq's published build, limited to the modes that
/// `extensions/code.cljs` in `logseq/og` bundles: name, aliases, mode and MIME type.
const MODES: &[(&str, &[&str], &str, &str)] = &[
    ("APL", &[], "apl", "text/apl"),
    ("PGP", &[], "asciiarmor", "application/pgp"),
    ("ASN.1", &[], "asn.1", "text/x-ttcn-asn"),
    ("Asterisk", &[], "asterisk", "text/x-asterisk"),
    ("Brainfuck", &[], "brainfuck", "text/x-brainfuck"),
    ("C", &[], "clike", "text/x-csrc"),
    ("C++", &["cpp"], "clike", "text/x-c++src"),
    ("Cobol", &[], "cobol", "text/x-cobol"),
    ("C#", &["csharp", "cs"], "clike", "text/x-csharp"),
    ("Clojure", &[], "clojure", "text/x-clojure"),
    ("ClojureScript", &[], "clojure", "text/x-clojurescript"),
    ("Closure Stylesheets (GSS)", &[], "css", "text/x-gss"),
    ("CMake", &[], "cmake", "text/x-cmake"),
    (
        "CoffeeScript",
        &["coffee", "coffee-script"],
        "coffeescript",
        "application/vnd.coffeescript",
    ),
    ("Common Lisp", &["lisp"], "commonlisp", "text/x-common-lisp"),
    ("Cypher", &[], "cypher", "application/x-cypher-query"),
    ("Cython", &[], "python", "text/x-cython"),
    ("Crystal", &[], "crystal", "text/x-crystal"),
    ("CSS", &[], "css", "text/css"),
    ("CQL", &[], "sql", "text/x-cassandra"),
    ("D", &[], "d", "text/x-d"),
    ("Dart", &[], "dart", "application/dart"),
    ("diff", &[], "diff", "text/x-diff"),
    ("Django", &[], "django", "text/x-django"),
    ("Dockerfile", &[], "dockerfile", "text/x-dockerfile"),
    ("DTD", &[], "dtd", "application/xml-dtd"),
    ("Dylan", &[], "dylan", "text/x-dylan"),
    ("EBNF", &[], "ebnf", "text/x-ebnf"),
    ("ECL", &[], "ecl", "text/x-ecl"),
    ("edn", &[], "clojure", "application/edn"),
    ("Eiffel", &[], "eiffel", "text/x-eiffel"),
    ("Elm", &[], "elm", "text/x-elm"),
    (
        "Embedded JavaScript",
        &[],
        "htmlembedded",
        "application/x-ejs",
    ),
    ("Embedded Ruby", &[], "htmlembedded", "application/x-erb"),
    ("Erlang", &[], "erlang", "text/x-erlang"),
    ("Esper", &[], "sql", "text/x-esper"),
    ("Factor", &[], "factor", "text/x-factor"),
    ("FCL", &[], "fcl", "text/x-fcl"),
    ("Forth", &[], "forth", "text/x-forth"),
    ("Fortran", &[], "fortran", "text/x-fortran"),
    ("F#", &["fsharp"], "mllike", "text/x-fsharp"),
    ("Gas", &[], "gas", "text/x-gas"),
    ("Gherkin", &[], "gherkin", "text/x-feature"),
    ("GitHub Flavored Markdown", &[], "gfm", "text/x-gfm"),
    ("Go", &[], "go", "text/x-go"),
    ("Groovy", &[], "groovy", "text/x-groovy"),
    ("HAML", &[], "haml", "text/x-haml"),
    ("Haskell", &[], "haskell", "text/x-haskell"),
    (
        "Haskell (Literate)",
        &[],
        "haskell-literate",
        "text/x-literate-haskell",
    ),
    ("Haxe", &[], "haxe", "text/x-haxe"),
    ("HXML", &[], "haxe", "text/x-hxml"),
    (
        "ASP.NET",
        &["asp", "aspx"],
        "htmlembedded",
        "application/x-aspx",
    ),
    ("HTML", &["xhtml"], "htmlmixed", "text/html"),
    ("HTTP", &[], "http", "message/http"),
    ("IDL", &[], "idl", "text/x-idl"),
    ("Pug", &["jade"], "pug", "text/x-pug"),
    ("Java", &[], "clike", "text/x-java"),
    (
        "Java Server Pages",
        &["jsp"],
        "htmlembedded",
        "application/x-jsp",
    ),
    (
        "JavaScript",
        &["ecmascript", "js", "node"],
        "javascript",
        "text/javascript",
    ),
    ("JSON", &["json5"], "javascript", "application/json"),
    ("JSON-LD", &["jsonld"], "javascript", "application/ld+json"),
    ("JSX", &[], "jsx", "text/jsx"),
    ("Jinja2", &[], "jinja2", "text/jinja2"),
    ("Julia", &["jl"], "julia", "text/x-julia"),
    ("Kotlin", &[], "clike", "text/x-kotlin"),
    ("LESS", &[], "css", "text/x-less"),
    ("LiveScript", &["ls"], "livescript", "text/x-livescript"),
    ("Lua", &[], "lua", "text/x-lua"),
    ("Markdown", &[], "markdown", "text/x-markdown"),
    ("mIRC", &[], "mirc", "text/mirc"),
    ("MariaDB SQL", &[], "sql", "text/x-mariadb"),
    ("Mathematica", &[], "mathematica", "text/x-mathematica"),
    ("Modelica", &[], "modelica", "text/x-modelica"),
    ("MUMPS", &[], "mumps", "text/x-mumps"),
    ("MS SQL", &[], "sql", "text/x-mssql"),
    ("mbox", &[], "mbox", "application/mbox"),
    ("MySQL", &[], "sql", "text/x-mysql"),
    ("Nginx", &[], "nginx", "text/x-nginx-conf"),
    ("NSIS", &[], "nsis", "text/x-nsis"),
    ("NTriples", &[], "ntriples", "application/n-triples"),
    (
        "Objective-C",
        &["objective-c", "objc"],
        "clike",
        "text/x-objectivec",
    ),
    (
        "Objective-C++",
        &["objective-c++", "objc++"],
        "clike",
        "text/x-objectivec++",
    ),
    ("OCaml", &[], "mllike", "text/x-ocaml"),
    ("Octave", &[], "octave", "text/x-octave"),
    ("Oz", &[], "oz", "text/x-oz"),
    ("Pascal", &[], "pascal", "text/x-pascal"),
    ("PEG.js", &[], "pegjs", "null"),
    ("Perl", &[], "perl", "text/x-perl"),
    ("PHP", &[], "php", "text/x-php"),
    ("Pig", &[], "pig", "text/x-pig"),
    ("PLSQL", &[], "sql", "text/x-plsql"),
    ("PostgreSQL", &[], "sql", "text/x-pgsql"),
    ("PowerShell", &[], "powershell", "application/x-powershell"),
    (
        "Properties files",
        &["ini", "properties"],
        "properties",
        "text/x-properties",
    ),
    ("ProtoBuf", &[], "protobuf", "text/x-protobuf"),
    ("Python", &[], "python", "text/x-python"),
    ("Puppet", &[], "puppet", "text/x-puppet"),
    ("Q", &[], "q", "text/x-q"),
    ("R", &["rscript"], "r", "text/x-rsrc"),
    ("reStructuredText", &["rst"], "rst", "text/x-rst"),
    ("RPM Changes", &[], "rpm", "text/x-rpm-changes"),
    ("RPM Spec", &[], "rpm", "text/x-rpm-spec"),
    (
        "Ruby",
        &["jruby", "macruby", "rake", "rb", "rbx"],
        "ruby",
        "text/x-ruby",
    ),
    ("Rust", &[], "rust", "text/x-rustsrc"),
    ("SAS", &[], "sas", "text/x-sas"),
    ("Sass", &[], "sass", "text/x-sass"),
    ("Scala", &[], "clike", "text/x-scala"),
    ("Scheme", &[], "scheme", "text/x-scheme"),
    ("SCSS", &[], "css", "text/x-scss"),
    ("Shell", &["bash", "sh", "zsh"], "shell", "text/x-sh"),
    ("Sieve", &[], "sieve", "application/sieve"),
    ("Slim", &[], "slim", "text/x-slim"),
    ("Smalltalk", &[], "smalltalk", "text/x-stsrc"),
    ("Smarty", &[], "smarty", "text/x-smarty"),
    ("Solr", &[], "solr", "text/x-solr"),
    ("SML", &[], "mllike", "text/x-sml"),
    ("Soy", &["closure template"], "soy", "text/x-soy"),
    ("SPARQL", &["sparul"], "sparql", "application/sparql-query"),
    (
        "Spreadsheet",
        &["excel", "formula"],
        "spreadsheet",
        "text/x-spreadsheet",
    ),
    ("SQL", &[], "sql", "text/x-sql"),
    ("SQLite", &[], "sql", "text/x-sqlite"),
    ("Squirrel", &[], "clike", "text/x-squirrel"),
    ("Stylus", &[], "stylus", "text/x-styl"),
    ("Swift", &[], "swift", "text/x-swift"),
    ("sTeX", &[], "stex", "text/x-stex"),
    ("LaTeX", &["tex"], "stex", "text/x-latex"),
    ("SystemVerilog", &[], "verilog", "text/x-systemverilog"),
    ("Tcl", &[], "tcl", "text/x-tcl"),
    ("Textile", &[], "textile", "text/x-textile"),
    ("TiddlyWiki", &[], "tiddlywiki", "text/x-tiddlywiki"),
    ("Tiki wiki", &[], "tiki", "text/tiki"),
    ("TOML", &[], "toml", "text/x-toml"),
    ("Tornado", &[], "tornado", "text/x-tornado"),
    ("troff", &[], "troff", "text/troff"),
    ("TTCN", &[], "ttcn", "text/x-ttcn"),
    ("TTCN_CFG", &[], "ttcn-cfg", "text/x-ttcn-cfg"),
    ("Turtle", &[], "turtle", "text/turtle"),
    (
        "TypeScript",
        &["ts"],
        "javascript",
        "application/typescript",
    ),
    ("TypeScript-JSX", &["tsx"], "jsx", "text/typescript-jsx"),
    ("Twig", &[], "twig", "text/x-twig"),
    ("Web IDL", &[], "webidl", "text/x-webidl"),
    ("VB.NET", &[], "vb", "text/x-vb"),
    ("VBScript", &[], "vbscript", "text/vbscript"),
    ("Velocity", &[], "velocity", "text/velocity"),
    ("Verilog", &[], "verilog", "text/x-verilog"),
    ("VHDL", &[], "vhdl", "text/x-vhdl"),
    ("Vue.js Component", &[], "vue", "script/x-vue"),
    ("XML", &["rss", "wsdl", "xsd"], "xml", "application/xml"),
    ("XQuery", &[], "xquery", "application/xquery"),
    ("Yacas", &[], "yacas", "text/x-yacas"),
    ("YAML", &["yml"], "yaml", "text/x-yaml"),
    ("Z80", &[], "z80", "text/x-z80"),
    ("mscgen", &[], "mscgen", "text/x-mscgen"),
    ("xu", &[], "mscgen", "text/x-xu"),
    ("msgenny", &[], "mscgen", "text/x-msgenny"),
    ("WebAssembly", &[], "wast", "text/webassembly"),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn src(source: &str) -> String {
        let parsed = lsdoc::parse_to_projection(source);
        let block = parsed
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Src { .. } | Block::Example { .. }))
            .expect("code block");
        code_text(source, block).unwrap()
    }

    #[test]
    fn nested_fence_loses_the_outline_indentation() {
        let source = "- a\n\t- b\n\t  ```rust\n\t  fn main() {\n\t      x();\n\t  }\n\t  ```\n";
        assert_eq!(src(source), "fn main() {\n    x();\n}");
    }

    #[test]
    fn top_level_fence_keeps_deeper_indentation() {
        let source = "- a\n  ~~~bash\n  echo hi\n    deeper\n  ~~~\n";
        assert_eq!(src(source), "echo hi\n  deeper");
    }

    #[test]
    fn short_and_blank_lines_lose_only_what_they_have() {
        let source = "- a\n\t  ```\n\t  one\n\n\t two\n\t  three\n\t  ```\n";
        assert_eq!(src(source), "one\n\ntwo\nthree");
    }

    #[test]
    fn fence_on_the_bullet_line_uses_the_bullet_indentation() {
        let source = "- a\n\t- ```sh\n\t  ls\n\t  ```\n";
        assert_eq!(src(source), "ls");
    }

    #[test]
    fn example_text_drops_the_trailing_newline() {
        let source = "- #+BEGIN_EXAMPLE\n  long line\n  #+END_EXAMPLE\n";
        assert_eq!(src(source), "long line");
    }

    #[test]
    fn other_blocks_have_no_code_text() {
        let parsed = lsdoc::parse_to_projection("- a\n");
        assert_eq!(code_text("- a\n", &parsed.blocks[0]), None);
    }

    fn mode(fence: &str) -> Option<&'static str> {
        code_language(fence).mode.map(|m| m.name)
    }

    #[test]
    fn languages_logseq_highlights() {
        for (fence, name) in [
            ("bash", "Shell"),
            ("sh", "Shell"),
            ("zsh", "Shell"),
            ("json", "JSON"),
            ("toml", "TOML"),
            ("python", "Python"),
            ("rust", "Rust"),
            ("yaml", "YAML"),
            ("yml", "YAML"),
            ("typescript", "TypeScript"),
            ("ts", "TypeScript"),
            ("markdown", "Markdown"),
            ("lua", "Lua"),
            ("javascript", "JavaScript"),
            ("dockerfile", "Dockerfile"),
            ("clojure", "Clojure"),
            ("xml", "XML"),
            ("edn", "Clojure"),
            ("cljs", "Clojure"),
            ("css", "CSS"),
            ("c", "C"),
            ("go", "Go"),
            ("haskell", "Haskell"),
            ("ini", "Properties files"),
            ("cpp", "C++"),
            ("java", "Java"),
            ("Rust", "Rust"),
        ] {
            assert_eq!(mode(fence), Some(name), "{fence}");
        }
    }

    #[test]
    fn languages_logseq_leaves_plain() {
        for fence in [
            "py",
            "rs",
            "md",
            "text",
            "nix",
            "jsonc",
            "mermaid",
            "calc",
            "vim",
            "env",
            "gitignore",
            "kdl",
            "tmux",
            "config",
            "bell",
            "nonsense",
        ] {
            assert_eq!(mode(fence), None, "{fence}");
        }
    }

    #[test]
    fn label_is_the_fence_language_in_lower_case() {
        assert_eq!(code_language("Bash").label.as_deref(), Some("bash"));
        assert_eq!(code_language("mermaid").label.as_deref(), Some("mermaid"));
        assert_eq!(
            code_language(""),
            CodeLanguage {
                label: None,
                mode: None
            }
        );
    }
}
