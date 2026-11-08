//! The entity types a page declares with its `logseq-entity::` property.
//!
//! A garden marks what kind of thing a page describes by naming a type page, such as
//! `logseq-entity:: [[Logseq/Entity/Podcast/Episode]]`. The value is read with Logseq's
//! property typing ([`crate::query_page::typed_value`]), so `[[A]], [[B]]` names two types, and
//! each name is matched as Logseq matches page names: without case, and through aliases when
//! the caller's lookup knows them. The authored value is read even when the garden hides the
//! property from readers with `:block-hidden-properties`.
//!
//! ```
//! use logseq_gardener_sdk::entity_types::has_entity_type;
//! use logseq_gardener_sdk::page_names::lookup_key;
//!
//! let properties = [(
//!     "logseq-entity".to_owned(),
//!     "[[Logseq/Entity/Podcast/Episode/GitP/A]], [[logseq/entity/podcast/episode]]".to_owned(),
//! )];
//! assert!(has_entity_type(&properties, "Logseq/Entity/Podcast/Episode", |name| lookup_key(name)));
//! assert!(!has_entity_type(&properties, "Logseq/Entity/Podcast", |name| lookup_key(name)));
//! ```
use std::collections::BTreeSet;

use crate::query_page::{property_key, typed_value};
use crate::query_script::Value;

/// The page property that names a page's entity types.
pub const ENTITY_PROPERTY: &str = "logseq-entity";

/// The types a page's `logseq-entity::` value names, each mapped through `resolve`.
///
/// `properties` are the page's authored properties as `(key, value)`; keys are compared as
/// Logseq stores them (lower case, `_` read as `-`), and the last value of a repeated key wins.
/// `resolve` turns a written name into the identity the caller compares, such as the route of
/// the page a name or alias selects, or [`lookup_key`](crate::page_names::lookup_key) of a name
/// no page holds.
pub fn entity_types(
    properties: &[(String, String)],
    resolve: impl Fn(&str) -> String,
) -> BTreeSet<String> {
    let Some((_, raw)) = properties
        .iter()
        .rev()
        .find(|(key, _)| property_key(key) == ENTITY_PROPERTY)
    else {
        return BTreeSet::new();
    };
    match typed_value(ENTITY_PROPERTY, raw) {
        Value::Set(names) => names
            .iter()
            .filter_map(|name| match name {
                Value::Str(name) => Some(resolve(name)),
                _ => None,
            })
            .collect(),
        _ => BTreeSet::new(),
    }
}

/// Whether a page's `logseq-entity::` value names `type_name` itself. A subtype such as
/// `Logseq/Entity/Podcast/Episode/GitP/A` alone does not match its parent type.
pub fn has_entity_type(
    properties: &[(String, String)],
    type_name: &str,
    resolve: impl Fn(&str) -> String,
) -> bool {
    let wanted = resolve(type_name);
    entity_types(properties, resolve).contains(&wanted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page_names::lookup_key;

    const EPISODE: &str = "Logseq/Entity/Podcast/Episode";

    fn props(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    fn by_key(name: &str) -> String {
        lookup_key(name)
    }

    #[test]
    fn matches_without_case() {
        let page = props(&[("logseq-entity", "[[logseq/ENTITY/podcast/episode]]")]);
        assert!(has_entity_type(&page, EPISODE, by_key));
        let page = props(&[("Logseq_Entity", "[[Logseq/Entity/Podcast/Episode]]")]);
        assert!(has_entity_type(&page, EPISODE, by_key));
    }

    #[test]
    fn matches_through_an_alias() {
        // The caller's lookup sends the alias and the type to the same page.
        let resolve = |name: &str| match lookup_key(name).as_str() {
            "episode" | "logseq/entity/podcast/episode" => "route-of-episode".to_owned(),
            other => other.to_owned(),
        };
        let page = props(&[("logseq-entity", "[[Episode]]")]);
        assert!(has_entity_type(&page, EPISODE, resolve));
    }

    #[test]
    fn reads_two_types_in_one_value() {
        let page = props(&[(
            "logseq-entity",
            "[[Logseq/Entity/Podcast/Episode/GitP/A]], [[Logseq/Entity/Podcast/Episode]]",
        )]);
        assert_eq!(
            entity_types(&page, by_key),
            BTreeSet::from([
                "logseq/entity/podcast/episode".to_owned(),
                "logseq/entity/podcast/episode/gitp/a".to_owned(),
            ])
        );
        assert!(has_entity_type(&page, EPISODE, by_key));
    }

    #[test]
    fn a_subtype_alone_does_not_match() {
        let page = props(&[("logseq-entity", "[[Logseq/Entity/Podcast/Episode/GitP/A]]")]);
        assert!(!has_entity_type(&page, EPISODE, by_key));
    }

    #[test]
    fn plain_text_and_other_properties_do_not_match() {
        assert!(!has_entity_type(
            &props(&[("logseq-entity", "Logseq/Entity/Podcast/Episode")]),
            EPISODE,
            by_key
        ));
        assert!(!has_entity_type(
            &props(&[("type", "[[Logseq/Entity/Podcast/Episode]]")]),
            EPISODE,
            by_key
        ));
        assert!(!has_entity_type(&[], EPISODE, by_key));
    }
}
