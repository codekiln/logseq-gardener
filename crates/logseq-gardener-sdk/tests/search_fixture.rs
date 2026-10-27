//! Runs `tests/search-fixture.json`, which `tests/search.test.cjs` also runs against the
//! browser engine, so the two engines cannot drift. The fixture also holds the files
//! the SDK writes for its records; the browser test queries those files.
use logseq_gardener_sdk::search::{
    BlockRecord, FileOptions, Group, PageRecord, QueryError, QueryOptions, Records, SearchIndex,
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

const ENGINE: &str = "lsg-search-engine-0000000000000000.mjs";

fn path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/search-fixture.json")
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_owned())
        .collect()
}

fn index(fixture: &Value) -> SearchIndex {
    let pages = fixture["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| PageRecord {
            title: p[0].as_str().unwrap().into(),
            id: p[1].as_str().unwrap().into(),
            aliases: strings(&p[2]),
        })
        .collect();
    let blocks = fixture["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(position, b)| BlockRecord {
            page: b[0].as_str().unwrap().into(),
            position,
            fragment: format!("source-block-{position}"),
            text: b[1].as_str().unwrap().into(),
            parents: strings(&b[2]),
        })
        .collect();
    SearchIndex::new(Records { pages, blocks })
}

fn results(index: &SearchIndex, query: &Value) -> Value {
    let page_id = |title: &str| {
        index
            .pages()
            .iter()
            .find(|p| p.title == title)
            .unwrap()
            .id
            .clone()
    };
    let options = QueryOptions {
        limit: query["limit"].as_u64().map_or(100, |n| n as usize),
        current_page: query["current_page"].as_str().map(page_id),
    };
    let blocks = |group: &Group<u32>| {
        let hits: Vec<_> = group
            .hits
            .iter()
            .map(|n| {
                let block = &index.blocks()[*n as usize];
                let page = index.pages().iter().find(|p| p.id == block.page).unwrap();
                format!("{}: {}", page.title, block.text)
            })
            .collect();
        json!({"count": group.count, "hits": hits})
    };
    match index.query(query["query"].as_str().unwrap(), &options) {
        Err(error) => json!({"error": match error {
            QueryError::InvalidQuery => "invalid-query",
            QueryError::TooManyTerms => "too-many-terms",
            QueryError::InvalidLimit => "invalid-limit",
        }}),
        Ok(results) => {
            let pages: Vec<_> = results
                .pages
                .hits
                .iter()
                .map(|hit| {
                    let title = &index.pages()[hit.page].title;
                    hit.alias
                        .as_ref()
                        .map_or(title.clone(), |alias| format!("{title} ← {alias}"))
                })
                .collect();
            json!({
                "error": null,
                "pages": {"count": results.pages.count, "hits": pages},
                "blocks": blocks(&results.blocks),
                "current_page": blocks(&results.current_page),
            })
        }
    }
}

#[test]
fn shared_fixture_matches_the_rust_query_and_files() {
    let mut fixture: Value = serde_json::from_str(&fs::read_to_string(path()).unwrap()).unwrap();
    let index = index(&fixture);
    let options = &fixture["file_options"];
    let files = index.files(
        ENGINE,
        &FileOptions {
            shard_budget: options["shard_budget"].as_u64().unwrap() as usize,
            chunk_size: options["chunk_size"].as_u64().unwrap() as usize,
        },
    );
    let published = json!({"manifest": files.manifest, "files": files.files});
    let actual: Vec<_> = fixture["queries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|query| results(&index, query))
        .collect();
    if std::env::var_os("LSG_UPDATE_SEARCH_FIXTURE").is_some() {
        for (query, expected) in fixture["queries"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .zip(actual)
        {
            query["expected"] = expected;
        }
        fixture["published"] = published;
        fs::write(
            path(),
            serde_json::to_string_pretty(&fixture).unwrap() + "\n",
        )
        .unwrap();
        return;
    }
    for (query, actual) in fixture["queries"].as_array().unwrap().iter().zip(actual) {
        assert_eq!(query["expected"], actual, "query {}", query["query"]);
    }
    assert_eq!(
        fixture["published"], published,
        "published files differ; regenerate the fixture"
    );
}
