//! A small EDN reader that keeps the byte offset of every form, so the Datalog and
//! function readers can name the form and where it is when they reject it.
use crate::query::{QueryError, QueryErrorKind};

/// One EDN form and where it starts in the text that was read.
#[derive(Clone, Debug, PartialEq)]
pub struct Form {
    pub offset: usize,
    pub end: usize,
    pub kind: FormKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FormKind {
    Nil,
    Bool(bool),
    Num(f64),
    Str(String),
    Keyword(String),
    Symbol(String),
    Vector(Vec<Form>),
    List(Vec<Form>),
    Map(Vec<(Form, Form)>),
    Set(Vec<Form>),
    /// A form the closed languages do not read (a regular expression, an anonymous
    /// function, a character, a tagged value or a quoted form), as written.
    Other(String),
}

impl Form {
    /// The form as the author wrote it, for messages: symbols, keywords and the text of
    /// other forms as is, collections by their opening bracket.
    pub fn name(&self) -> String {
        match &self.kind {
            FormKind::Symbol(s) => s.clone(),
            FormKind::Keyword(k) => format!(":{k}"),
            FormKind::Other(text) => text.clone(),
            FormKind::Nil => "nil".into(),
            FormKind::Bool(b) => b.to_string(),
            FormKind::Num(n) => n.to_string(),
            FormKind::Str(s) => format!("{s:?}"),
            FormKind::Vector(_) => "[".into(),
            FormKind::List(_) => "(".into(),
            FormKind::Map(_) => "{".into(),
            FormKind::Set(_) => "#{".into(),
        }
    }
}

fn invalid(message: &str, offset: usize) -> QueryError {
    QueryError {
        kind: QueryErrorKind::InvalidMap(message.to_owned()),
        offset,
    }
}

const MAX_DEPTH: usize = 64;

/// Read the one form in `text`. Offsets count from `base`, the position of `text` in the
/// block the caller is reading.
pub fn read_form(text: &str, base: usize) -> Result<Form, QueryError> {
    let mut reader = Reader {
        text: text.as_bytes(),
        source: text,
        pos: 0,
        base,
    };
    let form = reader
        .form(0)?
        .ok_or_else(|| invalid("missing form", base))?;
    reader.skip();
    if reader.pos < reader.text.len() {
        return Err(invalid("text after the form", base + reader.pos));
    }
    Ok(form)
}

struct Reader<'a> {
    text: &'a [u8],
    source: &'a str,
    pos: usize,
    base: usize,
}

impl Reader<'_> {
    fn skip(&mut self) {
        loop {
            while self
                .text
                .get(self.pos)
                .is_some_and(|c| c.is_ascii_whitespace() || *c == b',')
            {
                self.pos += 1;
            }
            if self.text.get(self.pos) == Some(&b';') {
                while self.text.get(self.pos).is_some_and(|c| *c != b'\n') {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    fn at(&self, pos: usize) -> usize {
        self.base + pos
    }

    fn form(&mut self, depth: usize) -> Result<Option<Form>, QueryError> {
        self.skip();
        while self.text.get(self.pos..self.pos + 2) == Some(b"#_") {
            self.pos += 2;
            self.form(depth + 1)?
                .ok_or_else(|| invalid("missing discarded form", self.at(self.pos)))?;
            self.skip();
        }
        let start = self.pos;
        let Some(&c) = self.text.get(self.pos) else {
            return Ok(None);
        };
        if depth > MAX_DEPTH {
            return Err(invalid("nesting exceeds 64 levels", self.at(start)));
        }
        self.pos += 1;
        let kind = match c {
            b'"' => FormKind::Str(self.string(start)?),
            b'(' => FormKind::List(self.items(b')', start, depth)?),
            b'[' => FormKind::Vector(self.items(b']', start, depth)?),
            b'{' => {
                let items = self.items(b'}', start, depth)?;
                if items.len() % 2 != 0 {
                    return Err(invalid("map has an unmatched key", self.at(start)));
                }
                let mut pairs = Vec::new();
                let mut items = items.into_iter();
                while let (Some(key), Some(value)) = (items.next(), items.next()) {
                    pairs.push((key, value));
                }
                FormKind::Map(pairs)
            }
            b'#' if self.text.get(self.pos) == Some(&b'{') => {
                self.pos += 1;
                FormKind::Set(self.items(b'}', start, depth)?)
            }
            b'#' if self.text.get(self.pos) == Some(&b'(') => {
                self.pos += 1;
                self.items(b')', start, depth)?;
                FormKind::Other("#(".into())
            }
            b'#' if self.text.get(self.pos) == Some(&b'"') => {
                self.pos += 1;
                self.string(start)?;
                FormKind::Other("#\"".into())
            }
            b')' | b']' | b'}' => return Err(invalid("unbalanced brackets", self.at(start))),
            b'\'' | b'`' | b'~' | b'@' | b'^' => {
                self.form(depth + 1)?
                    .ok_or_else(|| invalid("missing quoted form", self.at(self.pos)))?;
                FormKind::Other((c as char).to_string())
            }
            b'\\' => {
                self.atom_end();
                FormKind::Other(self.source[start..self.pos].to_owned())
            }
            _ => {
                self.atom_end();
                let atom = &self.source[start..self.pos];
                if atom.starts_with('#') && !atom.starts_with("##") {
                    self.form(depth + 1)?
                        .ok_or_else(|| invalid("missing tagged value", self.at(self.pos)))?;
                    FormKind::Other(atom.to_owned())
                } else {
                    atom_kind(atom)
                }
            }
        };
        Ok(Some(Form {
            offset: self.at(start),
            end: self.at(self.pos),
            kind,
        }))
    }

    fn items(&mut self, close: u8, start: usize, depth: usize) -> Result<Vec<Form>, QueryError> {
        let mut items = Vec::new();
        loop {
            self.skip();
            match self.text.get(self.pos) {
                None => return Err(invalid("unbalanced brackets", self.at(start))),
                Some(&b) if b == close => {
                    self.pos += 1;
                    return Ok(items);
                }
                Some(b')' | b']' | b'}') => {
                    return Err(invalid("unbalanced brackets", self.at(self.pos)));
                }
                Some(_) => {
                    let form = self.form(depth + 1)?.expect("text remains");
                    items.push(form);
                }
            }
        }
    }

    /// The text inside quotes, after the opening quote at `self.pos - 1`.
    fn string(&mut self, start: usize) -> Result<String, QueryError> {
        let mut out = String::new();
        let mut run = self.pos;
        loop {
            match self.text.get(self.pos) {
                None => return Err(invalid("unterminated string", self.at(start))),
                Some(b'\\') => {
                    out.push_str(&self.source[run..self.pos]);
                    let escaped = *self
                        .text
                        .get(self.pos + 1)
                        .ok_or_else(|| invalid("unterminated string", self.at(start)))?;
                    self.pos += 2;
                    match escaped {
                        b'n' => out.push('\n'),
                        b't' => out.push('\t'),
                        b'r' => out.push('\r'),
                        b'u' => {
                            let hex = self.source.get(self.pos..self.pos + 4).unwrap_or_default();
                            let code = u32::from_str_radix(hex, 16)
                                .map_err(|_| invalid("bad \\u escape", self.at(self.pos)))?;
                            out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                            self.pos += 4;
                        }
                        _ => {
                            // The escaped character may be multi-byte: copy it whole.
                            let from = self.pos - 1;
                            let ch = self.source[from..].chars().next().unwrap_or('\\');
                            out.push(ch);
                            self.pos = from + ch.len_utf8();
                        }
                    }
                    run = self.pos;
                }
                Some(b'"') => {
                    out.push_str(&self.source[run..self.pos]);
                    self.pos += 1;
                    return Ok(out);
                }
                Some(_) => self.pos += 1,
            }
        }
    }

    fn atom_end(&mut self) {
        while self
            .text
            .get(self.pos)
            .is_some_and(|c| !c.is_ascii_whitespace() && !b",{}[]()\";".contains(c))
        {
            self.pos += 1;
        }
    }
}

fn atom_kind(atom: &str) -> FormKind {
    match atom {
        "nil" => FormKind::Nil,
        "true" => FormKind::Bool(true),
        "false" => FormKind::Bool(false),
        _ => {
            if let Some(keyword) = atom.strip_prefix(':') {
                return FormKind::Keyword(keyword.to_owned());
            }
            let digits = atom.strip_prefix(['-', '+']).unwrap_or(atom);
            if digits.starts_with(|c: char| c.is_ascii_digit())
                && let Ok(number) = atom.parse::<f64>()
            {
                return FormKind::Num(number);
            }
            FormKind::Symbol(atom.to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_forms_with_offsets() {
        let form = read_form("[:a (f \"x\\ny\") 1.5 nil #{1}]", 10).unwrap();
        assert_eq!(form.offset, 10);
        let FormKind::Vector(items) = &form.kind else {
            panic!("{form:?}");
        };
        assert_eq!(items[0].kind, FormKind::Keyword("a".into()));
        assert_eq!(items[1].offset, 14);
        assert_eq!(items[2].kind, FormKind::Num(1.5));
        assert_eq!(items[3].kind, FormKind::Nil);
        assert!(matches!(items[4].kind, FormKind::Set(_)));
        let FormKind::List(call) = &items[1].kind else {
            panic!()
        };
        assert_eq!(call[1].kind, FormKind::Str("x\ny".into()));
    }

    #[test]
    fn other_forms_keep_their_text() {
        let form = read_form("(f #\"a+\" #(x) 'y \\a)", 0).unwrap();
        let FormKind::List(items) = form.kind else {
            panic!()
        };
        assert_eq!(items[1].kind, FormKind::Other("#\"".into()));
        assert_eq!(items[2].kind, FormKind::Other("#(".into()));
        assert_eq!(items[3].kind, FormKind::Other("'".into()));
        assert_eq!(items[4].kind, FormKind::Other("\\a".into()));
    }

    #[test]
    fn rejects_unbalanced_text_after_and_deep_nesting() {
        assert!(read_form("[1", 0).is_err());
        assert!(read_form("[1] 2", 0).is_err());
        assert!(read_form(&"[".repeat(100), 0).is_err());
    }
}
