//! Read the two reference settings without evaluating EDN or following symlinks.
use std::fs;
use std::io::Read;
use std::path::Path;

const MAX_BYTES: usize = 1_048_576;
const MAX_DEPTH: usize = 64;
const MAX_FORMS: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ReferenceConfig {
    pub threshold: i64,
    pub open_depth: i64,
}
impl Default for ReferenceConfig {
    fn default() -> Self {
        Self {
            threshold: 100,
            open_depth: 2,
        }
    }
}

#[derive(Debug)]
enum Form {
    Atom(String),
    Map(Vec<(Form, Form)>),
    Other,
}
struct Reader<'a> {
    input: &'a [u8],
    pos: usize,
    work: usize,
}
impl Reader<'_> {
    fn whitespace(&mut self) {
        loop {
            while self
                .input
                .get(self.pos)
                .is_some_and(|c| c.is_ascii_whitespace() || *c == b',')
            {
                self.pos += 1;
            }
            if self.input.get(self.pos) != Some(&b';') {
                break;
            }
            while self.input.get(self.pos).is_some_and(|c| *c != b'\n') {
                self.pos += 1;
            }
        }
    }

    fn string(&mut self) -> Result<(), &'static str> {
        loop {
            match self.input.get(self.pos).copied() {
                Some(b'\\') => {
                    self.pos += 1;
                    self.input.get(self.pos).ok_or("unterminated string")?;
                    self.pos += 1;
                }
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(());
                }
                Some(_) => self.pos += 1,
                None => return Err("unterminated string"),
            }
        }
    }

    fn collection(&mut self, close: u8, map: bool, depth: usize) -> Result<Form, &'static str> {
        // Retain only the outer map. Nested config values are read but never evaluated.
        let mut values = Vec::new();
        let mut count = 0;
        loop {
            self.whitespace();
            if self.input.get(self.pos) == Some(&close) {
                self.pos += 1;
                break;
            }
            if self.input.get(self.pos..self.pos + 2) == Some(b"#_") {
                self.pos += 2;
                self.form(depth + 1)?.ok_or("missing discarded form")?;
                continue;
            }
            let value = self.form(depth + 1)?.ok_or("unterminated collection")?;
            count += 1;
            if map && depth == 0 {
                values.push(value);
            }
        }
        if map && count % 2 != 0 {
            return Err("map has an unmatched key");
        }
        if !map || depth != 0 {
            return Ok(Form::Other);
        }
        let mut pairs = Vec::new();
        let mut values = values.into_iter();
        while let Some(key) = values.next() {
            pairs.push((key, values.next().unwrap()));
        }
        Ok(Form::Map(pairs))
    }

    fn form(&mut self, depth: usize) -> Result<Option<Form>, &'static str> {
        if depth > MAX_DEPTH {
            return Err("nesting exceeds 64 levels");
        }
        if self.work >= MAX_FORMS {
            return Err("form count exceeds 100000");
        }
        self.work += 1;
        self.whitespace();
        while self.input.get(self.pos..self.pos + 2) == Some(b"#_") {
            self.pos += 2;
            self.form(depth + 1)?.ok_or("missing discarded form")?;
            self.whitespace();
        }
        let Some(c) = self.input.get(self.pos).copied() else {
            return Ok(None);
        };
        self.pos += 1;
        if c == b'"' {
            self.string()?;
            return Ok(Some(Form::Other));
        }
        if matches!(c, b'{' | b'[' | b'(') {
            let close = match c {
                b'{' => b'}',
                b'[' => b']',
                _ => b')',
            };
            return self.collection(close, c == b'{', depth).map(Some);
        }
        if matches!(c, b'}' | b']' | b')') {
            return Err("unexpected closing delimiter");
        }
        // Sets have collection semantics, never map key/value semantics.
        if c == b'#' && self.input.get(self.pos) == Some(&b'{') {
            self.pos += 1;
            return self.collection(b'}', false, depth).map(Some);
        }
        let start = self.pos - 1;
        if c == b'\\'
            && self
                .input
                .get(self.pos)
                .is_some_and(|b| b"{}[]()\";".contains(b))
        {
            self.pos += 1;
        } else {
            while self
                .input
                .get(self.pos)
                .is_some_and(|c| !c.is_ascii_whitespace() && !b",{}[]()\";".contains(c))
            {
                self.pos += 1;
            }
        }
        let atom =
            std::str::from_utf8(&self.input[start..self.pos]).map_err(|_| "invalid UTF-8")?;
        if atom.starts_with("##") {
            // EDN symbolic floating-point values are complete scalar forms.
            return match atom {
                "##Inf" | "##-Inf" | "##NaN" => Ok(Some(Form::Other)),
                _ => Err("unsupported symbolic value"),
            };
        }
        if atom.starts_with('#') {
            // Tagged values (including #inst and #uuid) are consumed as one form.
            // Reading the payload does not invoke a tag reader or evaluate code.
            self.form(depth + 1)?.ok_or("missing tagged value")?;
            return Ok(Some(Form::Other));
        }
        Ok(Some(Form::Atom(atom.to_owned())))
    }
}

fn defaults(reason: &str) -> (ReferenceConfig, Vec<String>) {
    (
        ReferenceConfig::default(),
        vec![format!("Reference settings use defaults: {reason}")],
    )
}

pub(super) fn parse(text: &str) -> (ReferenceConfig, Vec<String>) {
    if text.len() > MAX_BYTES {
        return defaults("config.edn exceeds 1 MiB");
    }
    let mut result = ReferenceConfig::default();
    let mut reader = Reader {
        input: text.as_bytes(),
        pos: 0,
        work: 0,
    };
    let pairs = match reader.form(0) {
        Ok(Some(Form::Map(pairs))) => pairs,
        Ok(_) => return defaults("config.edn must contain an EDN map"),
        Err(reason) => return defaults(reason),
    };
    match reader.form(0) {
        Ok(None) => {}
        Ok(Some(_)) => return defaults("extra EDN config content"),
        Err(reason) => return defaults(reason),
    }
    let mut diagnostics = Vec::new();
    for (key, value) in pairs {
        let Form::Atom(key) = key else {
            continue;
        };
        let destination = match key.as_str() {
            ":ref/linked-references-collapsed-threshold" => &mut result.threshold,
            ":ref/default-open-blocks-level" => &mut result.open_depth,
            _ => continue,
        };
        if let Form::Atom(value) = value
            && let Ok(value) = value.parse::<i64>()
        {
            *destination = value;
        } else {
            diagnostics.push(format!(
                "Reference setting {key} is not a supported integer; using its default"
            ));
            *destination = if key.ends_with("threshold") { 100 } else { 2 };
        }
    }
    (result, diagnostics)
}

pub(super) fn read(root: &Path) -> (ReferenceConfig, Vec<String>) {
    let directory = root.join("logseq");
    let path = directory.join("config.edn");
    for component in [&directory, &path] {
        match fs::symlink_metadata(component) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return (ReferenceConfig::default(), Vec::new());
            }
            Ok(meta) if !meta.file_type().is_symlink() => {}
            _ => return defaults("config path is unavailable or a symlink"),
        }
    }
    let Ok(meta) = fs::metadata(&path) else {
        return defaults("cannot read config.edn");
    };
    if !meta.is_file() {
        return defaults("config.edn must be a regular file");
    }
    if meta.len() > MAX_BYTES as u64 {
        return defaults("config.edn exceeds 1 MiB");
    }
    let mut content = Vec::new();
    let read = fs::File::open(&path)
        .and_then(|file| file.take(MAX_BYTES as u64 + 1).read_to_end(&mut content));
    if read.is_err() {
        return defaults("cannot read config.edn");
    }
    if content.len() > MAX_BYTES {
        return defaults("config.edn exceeds 1 MiB");
    }
    match std::str::from_utf8(&content) {
        Ok(text) => parse(text),
        Err(_) => defaults("cannot read config.edn as UTF-8"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_and_signed_integer_overrides_are_preserved() {
        let (config, diagnostics) = parse(
            "{:ref/linked-references-collapsed-threshold 0 :ref/default-open-blocks-level -3}",
        );
        assert_eq!(
            config,
            ReferenceConfig {
                threshold: 0,
                open_depth: -3
            }
        );
        assert!(diagnostics.is_empty());
        assert_eq!(parse("{:ref/default-open-blocks-level +4}").0.open_depth, 4);
    }

    #[test]
    fn comments_strings_nested_and_discarded_forms_do_not_change_settings() {
        let input = r##"{; :ref/default-open-blocks-level 99
            :note ":ref/default-open-blocks-level 8"
            :nested {:ref/default-open-blocks-level 9}
            #_ :ref/linked-references-collapsed-threshold #_ 0
            :ref/default-open-blocks-level 2 #_ {:ignored true}}
            #_ {:ref/default-open-blocks-level 99}"##;
        let (config, errors) = parse(input);
        assert_eq!(config, ReferenceConfig::default());
        assert!(errors.is_empty(), "{errors:?}");
        // The outer discard consumes the next readable form after the inner discard.
        assert_eq!(
            parse("{#_ #_ :discarded 99 :ref/default-open-blocks-level 4}")
                .0
                .open_depth,
            4
        );
    }

    #[test]
    fn common_edn_values_do_not_consume_neighboring_settings() {
        let input = r##"{:set #{:a :b :c} :empty #{} :list (a b c)
            :date #inst "2026-10-23T00:00:00Z" :id #uuid "not-evaluated"
            :custom #app/data {:nested #{:one}} :infinite ##Inf :negative ##-Inf :nan ##NaN
            :ratio 1/2 :big 1N :decimal 2M :boolean true :nil nil
            :characters [\; \" \} \newline] :escaped "quote: \" slash: \\"
            :ref/default-open-blocks-level 5 :ref/linked-references-collapsed-threshold 7}"##;
        let (config, diagnostics) = parse(input);
        assert_eq!(
            config,
            ReferenceConfig {
                threshold: 7,
                open_depth: 5
            }
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn malformed_and_non_integer_settings_use_defaults_with_diagnostics() {
        for input in [
            r#"{:ref/default-open-blocks-level "2"}"#,
            "{:ref/default-open-blocks-level [2]}",
            "{:ref/default-open-blocks-level ##Inf}",
            "{:ref/default-open-blocks-level 9223372036854775808}",
            "{:ref/default-open-blocks-level 2",
            "{} {}",
            "{:a}",
            "{:x [)}",
            "{:x #inst}",
            "{:x #_}",
            r#"{:x "unterminated}"#,
        ] {
            let (config, diagnostics) = parse(input);
            assert_eq!(config, ReferenceConfig::default(), "{input}");
            assert!(!diagnostics.is_empty(), "{input}");
        }
        // A bad value only defaults its own key.
        assert_eq!(
            parse(
                "{:ref/default-open-blocks-level nil :ref/linked-references-collapsed-threshold 9}"
            )
            .0
            .threshold,
            9
        );
    }

    #[test]
    fn reader_enforces_size_depth_and_form_limits() {
        assert!(parse(&" ".repeat(MAX_BYTES + 1)).1[0].contains("1 MiB"));
        let deep = format!(
            "{{:x {}0{}}}",
            "[".repeat(MAX_DEPTH + 1),
            "]".repeat(MAX_DEPTH + 1)
        );
        assert!(parse(&deep).1[0].contains("64 levels"));
        let work = format!("{{:x [{}]}}", "0 ".repeat(MAX_FORMS));
        assert!(parse(&work).1[0].contains("100000"));
        let boundary = format!("{{}}{}", " ".repeat(MAX_BYTES - 2));
        assert!(parse(&boundary).1.is_empty());
    }

    #[test]
    fn read_handles_absent_invalid_and_oversized_files() {
        let root = std::env::temp_dir().join(format!(
            "gardener-reference-config-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&root).unwrap();
        assert_eq!(read(&root), (ReferenceConfig::default(), vec![]));
        fs::create_dir(root.join("logseq")).unwrap();
        let path = root.join("logseq/config.edn");
        fs::write(&path, "{:ref/default-open-blocks-level 0}").unwrap();
        assert_eq!(read(&root).0.open_depth, 0);
        fs::write(&path, [0xff]).unwrap();
        assert!(read(&root).1[0].contains("UTF-8"));
        fs::write(&path, vec![b' '; MAX_BYTES + 1]).unwrap();
        assert!(read(&root).1[0].contains("1 MiB"));
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(read(&root).1[0].contains("regular file"));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn read_refuses_symlinked_config_and_logseq_directory() {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!(
            "gardener-reference-symlink-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(root.join("logseq")).unwrap();
        let real = root.join("real.edn");
        fs::write(&real, "{:ref/default-open-blocks-level 9}").unwrap();
        symlink(&real, root.join("logseq/config.edn")).unwrap();
        assert!(read(&root).1[0].contains("symlink"));
        fs::remove_file(root.join("logseq/config.edn")).unwrap();
        fs::remove_dir(root.join("logseq")).unwrap();
        fs::create_dir(root.join("real-logseq")).unwrap();
        symlink(root.join("real-logseq"), root.join("logseq")).unwrap();
        assert!(read(&root).1[0].contains("symlink"));
        fs::remove_dir_all(root).unwrap();
    }
}
