//! A `:view` function's result as a tree of elements and text. The tree holds no HTML:
//! the publisher decides which tags, attributes and URLs it writes.
use std::fmt;

use crate::query_script::{Value, number_text};

/// One node of a view's output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Hiccup {
    Text(String),
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<Hiccup>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HiccupError {
    /// A form the build does not read, such as a tag with `.` or `#` shorthand.
    Unsupported(String),
    /// A value that React would refuse to draw.
    Invalid(String),
}

impl fmt::Display for HiccupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HiccupError::Unsupported(form) => {
                write!(
                    f,
                    "the published site does not support the view form ({form})"
                )
            }
            HiccupError::Invalid(message) => f.write_str(message),
        }
    }
}

const MAX_DEPTH: usize = 128;

/// Turn the value a view returned into nodes. A vector whose first element is a string or
/// a keyword is an element; any other sequence is flattened; `nil`, `true` and `false`
/// are skipped; numbers are text.
pub fn query_hiccup(value: &Value) -> Result<Vec<Hiccup>, HiccupError> {
    let mut nodes = Vec::new();
    node(value, &mut nodes, 0)?;
    Ok(nodes)
}

fn node(value: &Value, out: &mut Vec<Hiccup>, depth: usize) -> Result<(), HiccupError> {
    if depth > MAX_DEPTH {
        return Err(HiccupError::Invalid("the view nests too deeply".into()));
    }
    match value {
        Value::Nil | Value::Bool(_) => {}
        Value::Str(text) => out.push(Hiccup::Text(text.to_string())),
        Value::Num(n) => out.push(Hiccup::Text(number_text(*n))),
        Value::Keyword(_) => out.push(Hiccup::Text(value.to_text())),
        Value::Vector(items) if is_element(items) => out.push(element(items, depth)?),
        Value::Vector(items) | Value::List(items) | Value::Set(items) => {
            for item in items.iter() {
                node(item, out, depth + 1)?;
            }
        }
        Value::Map(_) | Value::Row(_) => {
            return Err(HiccupError::Invalid(format!(
                "Objects are not valid as a React child (found: {})",
                value.pr_str()
            )));
        }
        Value::Fn(_) | Value::Builtin(_) => {
            return Err(HiccupError::Invalid(
                "Functions are not valid as a React child".into(),
            ));
        }
    }
    Ok(())
}

fn is_element(items: &[Value]) -> bool {
    matches!(items.first(), Some(Value::Str(_) | Value::Keyword(_)))
}

fn element(items: &[Value], depth: usize) -> Result<Hiccup, HiccupError> {
    let tag = match &items[0] {
        Value::Str(s) | Value::Keyword(s) => s.to_string(),
        _ => unreachable!("checked by is_element"),
    };
    if tag.is_empty() || tag.contains(['.', '#']) {
        return Err(HiccupError::Unsupported(tag));
    }
    let mut rest = &items[1..];
    let mut attrs = Vec::new();
    if let Some(Value::Map(pairs)) = rest.first() {
        for (key, value) in pairs.iter() {
            let name = match key {
                Value::Keyword(k) | Value::Str(k) => k.to_string(),
                other => {
                    return Err(HiccupError::Invalid(format!(
                        "an attribute name must be a keyword or a string, not {}",
                        other.pr_str()
                    )));
                }
            };
            match value {
                Value::Nil | Value::Bool(false) => {}
                Value::Bool(true) => attrs.push((name, String::new())),
                Value::Vector(parts) | Value::List(parts) | Value::Set(parts)
                    if name == "class" =>
                {
                    let classes: Vec<String> = parts.iter().map(Value::to_text).collect();
                    attrs.push((name, classes.join(" ")));
                }
                other => attrs.push((name, other.to_text())),
            }
        }
        rest = &rest[1..];
    }
    let mut children = Vec::new();
    for child in rest {
        node(child, &mut children, depth + 1)?;
    }
    Ok(Hiccup::Element {
        tag,
        attrs,
        children,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_form::read_form;
    use crate::query_script::{Meter, parse_script};

    fn view(source: &str, args: Vec<Value>) -> Result<Vec<Hiccup>, HiccupError> {
        let script = parse_script(&read_form(source, 0).unwrap()).unwrap();
        query_hiccup(&script.call(args, &mut Meter::default()).unwrap())
    }

    fn text(s: &str) -> Hiccup {
        Hiccup::Text(s.into())
    }

    #[test]
    fn link_list_tree() {
        let tree = view(
            "(fn [] [\"ul\" [\"li\" [\"a\" {:href \"#/page/x\"} \"X\"]]])",
            vec![],
        )
        .unwrap();
        assert_eq!(
            tree,
            [Hiccup::Element {
                tag: "ul".into(),
                attrs: vec![],
                children: vec![Hiccup::Element {
                    tag: "li".into(),
                    attrs: vec![],
                    children: vec![Hiccup::Element {
                        tag: "a".into(),
                        attrs: vec![("href".into(), "#/page/x".into())],
                        children: vec![text("X")],
                    }],
                }],
            }]
        );
    }

    #[test]
    fn mapped_rows_flatten_and_nil_false_are_skipped() {
        let tree = view(
            "(fn [rows] (into [\"ul\"] (map (fn [r] [\"li\" r nil false 3 1.5 :k]) rows)))",
            vec![Value::vector(vec![Value::str("a"), Value::str("b")])],
        )
        .unwrap();
        let Hiccup::Element { children, .. } = &tree[0] else {
            panic!()
        };
        assert_eq!(children.len(), 2);
        let Hiccup::Element { children, .. } = &children[0] else {
            panic!()
        };
        assert_eq!(children, &[text("a"), text("3"), text("1.5"), text(":k")]);
        // A sequence that is not an element flattens in place.
        let tree = view(
            "(fn [] [\"p\" (map str [\"a\" \"b\"]) [[\"b\" \"c\"]]])",
            vec![],
        )
        .unwrap();
        let Hiccup::Element { children, .. } = &tree[0] else {
            panic!()
        };
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn keyword_tags_attributes_and_classes() {
        let tree = view(
            "(fn [] [:div {:class [\"a\" \"b\"] :hidden true :title nil \"data-x\" 5} \"t\"])",
            vec![],
        )
        .unwrap();
        assert_eq!(
            tree,
            [Hiccup::Element {
                tag: "div".into(),
                attrs: vec![
                    ("class".into(), "a b".into()),
                    ("hidden".into(), String::new()),
                    ("data-x".into(), "5".into()),
                ],
                children: vec![text("t")],
            }]
        );
    }

    #[test]
    fn shorthand_tags_and_objects_are_errors() {
        assert_eq!(
            view("(fn [] [:div.card \"x\"])", vec![]).unwrap_err(),
            HiccupError::Unsupported("div.card".into())
        );
        assert_eq!(
            view("(fn [] [\"p#id\" \"x\"])", vec![]).unwrap_err(),
            HiccupError::Unsupported("p#id".into())
        );
        assert!(matches!(
            view("(fn [] [\"p\" {:a 1} {:b 2}])", vec![]).unwrap_err(),
            HiccupError::Invalid(_)
        ));
    }
}
