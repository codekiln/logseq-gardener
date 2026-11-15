//! End-to-end tests of go to definition, find references and hover in `lsg lsp`.

#[path = "lsp/client.rs"]
mod client;

use client::{Client, Garden, Options};
use serde_json::{Value, json};

const APPLE: &str = "pages/Fruit___Apple.md";
const PEAR: &str = "pages/Fruit___Pear.md";
const FRUIT: &str = "pages/Fruit.md";
const RECIPES: &str = "pages/Recipes.md";
const JANUARY_2: &str = "journals/2026_01_02.md";
const JANUARY_3: &str = "journals/2026_01_03.md";

const START: &str = r#"{"start":{"line":0,"character":0},"end":{"line":0,"character":0}}"#;

/// The answer to a request at the first `needle` of a file, three characters in.
fn ask(client: &mut Client, garden: &Garden, method: &str, file: &str, needle: &str) -> Value {
    let params = client.at(&garden.uri(file), &garden.read(file), needle, 3);
    client.request(method, params)
}

/// Where go to definition leads from the first `needle` of a file.
fn definition(client: &mut Client, garden: &Garden, file: &str, needle: &str) -> Value {
    ask(client, garden, "textDocument/definition", file, needle)
}

/// The one location in a definition answer: its file relative to the garden and its range.
fn one(garden: &Garden, answer: &Value) -> (String, Value) {
    let locations = answer.as_array().unwrap_or_else(|| panic!("{answer}"));
    assert_eq!(locations.len(), 1, "{answer}");
    (
        relative(garden, &locations[0]),
        locations[0]["range"].clone(),
    )
}

fn relative(garden: &Garden, location: &Value) -> String {
    let path = client::uri_to_path(location["uri"].as_str().unwrap());
    path.strip_prefix(&garden.root)
        .unwrap_or_else(|_| panic!("{} is outside the garden", path.display()))
        .to_string_lossy()
        .replace('\\', "/")
}

/// Each location of a references answer as its file and the text of its range.
fn places(client: &Client, garden: &Garden, answer: &Value) -> Vec<(String, String)> {
    answer
        .as_array()
        .unwrap_or_else(|| panic!("{answer}"))
        .iter()
        .map(|location| {
            let file = relative(garden, location);
            let text = garden.read(&file);
            let start = client::offset_of(&text, &location["range"]["start"], client.utf8);
            let end = client::offset_of(&text, &location["range"]["end"], client.utf8);
            (file, text[start..end].to_owned())
        })
        .collect()
}

fn references(
    client: &mut Client,
    garden: &Garden,
    file: &str,
    needle: &str,
    declaration: bool,
) -> Vec<(String, String)> {
    let mut params = client.at(&garden.uri(file), &garden.read(file), needle, 3);
    params["context"] = json!({"includeDeclaration": declaration});
    let answer = client.request("textDocument/references", params);
    places(client, garden, &answer)
}

fn place(file: &str, text: &str) -> (String, String) {
    (file.to_owned(), text.to_owned())
}

fn hover(client: &mut Client, garden: &Garden, file: &str, needle: &str) -> Value {
    ask(client, garden, "textDocument/hover", file, needle)
}

#[test]
fn the_server_advertises_definition_references_and_hover() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());
    let capabilities = &client.initialize_result["capabilities"];
    assert_eq!(capabilities["definitionProvider"], true);
    assert_eq!(capabilities["referencesProvider"], true);
    assert_eq!(capabilities["hoverProvider"], true);
    // The custom request from the server's start stays.
    let params = client.at(&garden.uri(RECIPES), &garden.read(RECIPES), "[[Malus]]", 3);
    assert_eq!(client.request("lsg/occurrence", params)["kind"], "pageLink");
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn definition_opens_the_page_each_kind_of_link_names() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());
    let start: Value = serde_json::from_str(START).unwrap();
    for (file, needle, target) in [
        // A page link to a namespace page, in any letter case.
        (RECIPES, "[[Fruit/Apple]] and", APPLE),
        (JANUARY_2, "[[fruit/apple]]", APPLE),
        // The parent of a namespace.
        (PEAR, "[[Fruit]]", FRUIT),
        // An alias leads to the page that declares it.
        (RECIPES, "[[Malus]]", APPLE),
        (JANUARY_2, "[[Pyrus]]", PEAR),
        // Tags.
        (FRUIT, "#recipes", RECIPES),
        (RECIPES, "#[[multi word]]", "pages/multi word.md"),
        // A page link as the target of a labelled link, and a page embed.
        (RECIPES, "[the pear page]([[Fruit/Pear]])", PEAR),
        (RECIPES, "{{embed [[Fruit/Pear]]}}", PEAR),
        // A journal date in the title format and in another format Logseq accepts.
        (RECIPES, "[[Jan 2nd, 2026]]", JANUARY_2),
        (RECIPES, "[[2026-01-03]]", JANUARY_3),
        // Page names in property values: a link, and a bare name in `alias::`.
        (APPLE, "[[Fruit]], sweet", FRUIT),
        (APPLE, "Malus, [[Pomme]]", APPLE),
        // A page, a tag, a property value and a journal that have no file yet.
        (RECIPES, "[[Fruit/Quince]]", "pages/Fruit___Quince.md"),
        (RECIPES, "#sweet", "pages/sweet.md"),
        (APPLE, "sweet", "pages/sweet.md"),
        (RECIPES, "[[Category]]", "pages/Category.md"),
    ] {
        let answer = definition(&mut client, &garden, file, needle);
        assert_eq!(
            one(&garden, &answer),
            (target.to_owned(), start.clone()),
            "{needle} in {file}"
        );
    }
    assert!(!garden.path("pages/Fruit___Quince.md").exists());

    // Plain text and a property key lead nowhere.
    assert_eq!(
        definition(&mut client, &garden, RECIPES, "Peel four"),
        Value::Null
    );
    assert_eq!(
        definition(&mut client, &garden, RECIPES, "serves::"),
        json!([])
    );
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn definition_opens_the_block_a_reference_or_embed_names() {
    let garden = Garden::copy();
    let apple = garden.read(APPLE);
    let journal = garden.read(JANUARY_2);
    for utf8 in [true, false] {
        let options = Options {
            utf8,
            ..Options::default()
        };
        let mut client = Client::start(&garden, options);
        // The block's first line, from its dash to the end of the line.
        let line = |text: &str, needle: &str| {
            json!({
                "start": client::position_of(text, needle, 0, utf8),
                "end": client::position_of(text, needle, needle.len(), utf8),
            })
        };
        let embed = "{{embed ((66f1a2b0-0000-4000-8000-000000000002))}}";
        assert_eq!(
            one(&garden, &definition(&mut client, &garden, RECIPES, embed)),
            (
                APPLE.to_owned(),
                line(&apple, "- Store apart from [[Fruit/Pear]].")
            )
        );
        let reference = "((66f1a2b0-0000-4000-8000-000000000004))";
        assert_eq!(
            one(
                &garden,
                &definition(&mut client, &garden, JANUARY_3, reference)
            ),
            (
                JANUARY_2.to_owned(),
                line(&journal, "- Baked the pear tart from [[Recipes]].")
            )
        );
        // A reference from another page.
        let shared = "((66f1a2b0-0000-4000-8000-000000000001))";
        assert_eq!(
            one(&garden, &definition(&mut client, &garden, PEAR, shared)).0,
            APPLE
        );
        // A reference to a block that is gone leads nowhere.
        let gone = "((66f1a2b0-0000-4000-8000-00000000dead))";
        assert_eq!(definition(&mut client, &garden, RECIPES, gone), json!([]));
        // A link after an emoji on its line, in both encodings.
        assert_eq!(
            one(
                &garden,
                &definition(&mut client, &garden, RECIPES, "[[Fruit/Apple]] and")
            )
            .0,
            APPLE
        );
        assert_eq!(client.shutdown().code(), Some(0));
    }
}

#[test]
fn definition_names_the_file_logseq_would_create() {
    // The legacy file name format and a journal file name format of the garden's own.
    let garden = Garden::empty();
    garden.write(
        "logseq/config.edn",
        "{:journal/file-name-format \"yyyy-MM-dd\"}\n",
    );
    garden.write(
        "pages/Notes.md",
        "- [[Fruit/Quince]] and [[What is 50%?]] on [[Jan 4th, 2026]] or [[2026_01_05]]\n",
    );
    let mut client = Client::start(&garden, Options::default());
    for (needle, target) in [
        ("[[Fruit/Quince]]", "pages/Fruit%2FQuince.md"),
        ("[[What is 50%?]]", "pages/What is 50%25%3F.md"),
        ("[[Jan 4th, 2026]]", "journals/2026-01-04.md"),
        ("[[2026_01_05]]", "journals/2026-01-05.md"),
    ] {
        let answer = definition(&mut client, &garden, "pages/Notes.md", needle);
        assert_eq!(one(&garden, &answer).0, target, "{needle}");
    }

    // Once the file exists, the same link leads to it.
    garden.write("journals/2026-01-04.md", "- A new day.\n");
    client.files_changed(&[(&garden.uri("journals/2026-01-04.md"), 1)]);
    let answer = definition(&mut client, &garden, "pages/Notes.md", "[[Jan 4th, 2026]]");
    assert_eq!(one(&garden, &answer).0, "journals/2026-01-04.md");
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn references_list_every_place_that_names_a_page_or_one_of_its_aliases() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());
    let linked = [
        place(JANUARY_2, "[[fruit/apple]]"),
        place(FRUIT, "[[Fruit/Apple]]"),
        place(RECIPES, "[[Malus]]"),
        place(RECIPES, "[[Fruit/Apple]]"),
    ];
    // From a link that writes an alias of the page.
    assert_eq!(
        references(&mut client, &garden, RECIPES, "[[Malus]]", false),
        linked
    );
    // With the declaration: the page's file and the names its `alias::` gives it.
    let mut declared = vec![
        place(APPLE, ""),
        place(APPLE, "Malus"),
        place(APPLE, "[[Pomme]]"),
    ];
    declared.extend(linked.clone());
    assert_eq!(
        references(&mut client, &garden, RECIPES, "[[Malus]]", true),
        declared
    );

    // From plain text: the places that link to the current page.
    assert_eq!(
        references(&mut client, &garden, APPLE, "An apple is red", false),
        linked
    );
    assert_eq!(
        references(&mut client, &garden, PEAR, "A pear ripens", false),
        [
            place(JANUARY_2, "[[Pyrus]]"),
            place(FRUIT, "[[Fruit/Pear]]"),
            place(FRUIT, "[[Pyrus]]"),
            place(APPLE, "[[Fruit/Pear]]"),
            place(RECIPES, "[the pear page]([[Fruit/Pear]])"),
            place(RECIPES, "{{embed [[Fruit/Pear]]}}"),
        ]
    );

    // A page without a file has references and no declaration.
    assert_eq!(
        references(&mut client, &garden, RECIPES, "#sweet", true),
        [
            place(JANUARY_2, "#sweet"),
            place(APPLE, "sweet"),
            place(RECIPES, "#sweet"),
        ]
    );

    // An unsaved edit adds a reference.
    let uri = garden.uri(FRUIT);
    let text = garden.read(FRUIT);
    client.open(&uri, &text);
    client.replace(&uri, 2, &text, "trees and bushes", "trees like [[Pomme]]");
    let found = client.request(
        "textDocument/references",
        json!({
            "textDocument": {"uri": garden.uri(APPLE)},
            "position": {"line": 3, "character": 4},
            "context": {"includeDeclaration": false},
        }),
    );
    assert_eq!(found.as_array().unwrap().len(), linked.len() + 1);
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn references_list_every_reference_and_embed_of_a_block() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());
    let id = "66f1a2b0-0000-4000-8000-000000000002";
    let embed = format!("{{{{embed (({id}))}}}}");
    assert_eq!(
        references(&mut client, &garden, RECIPES, &embed, false),
        [place(RECIPES, &embed)]
    );
    // The declaration is the `id::` value of the block, and the cursor can be on it.
    assert_eq!(
        references(&mut client, &garden, APPLE, id, true),
        [place(APPLE, id), place(RECIPES, &embed)]
    );
    // A property key lists the lines that write it.
    assert_eq!(
        references(&mut client, &garden, RECIPES, "type:: recipe", false),
        [
            place(FRUIT, "type"),
            place(RECIPES, "type"),
            place(RECIPES, "type"),
            place(RECIPES, "type"),
        ]
    );
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn hover_shows_a_block_and_a_page_as_markdown() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());
    let recipes = garden.read(RECIPES);

    let embed = "{{embed ((66f1a2b0-0000-4000-8000-000000000002))}}";
    let shown = hover(&mut client, &garden, RECIPES, embed);
    assert_eq!(shown["contents"]["kind"], "markdown");
    assert_eq!(
        shown["contents"]["value"],
        "**Fruit/Apple** › Keeps for months in a cool cellar.\n\nStore apart from [[Fruit/Pear]].\n\n1 reference"
    );
    assert_eq!(
        shown["range"],
        json!({
            "start": client::position_of(&recipes, embed, 0, true),
            "end": client::position_of(&recipes, embed, embed.len(), true),
        })
    );

    // A block with a property Logseq shows; its `id::` line is left out.
    let shown = hover(
        &mut client,
        &garden,
        PEAR,
        "((66f1a2b0-0000-4000-8000-000000000001))",
    );
    assert_eq!(
        shown["contents"]["value"],
        "**Fruit/Apple**\n\nAn apple is red or green.\ncolour:: red\n\n1 reference"
    );

    let shown = hover(&mut client, &garden, RECIPES, "[[Malus]]");
    assert_eq!(
        shown["contents"]["value"],
        "**Fruit/Apple**\n\nAliases: Malus, Pomme\n\n4 references\n\n---\n\n- An apple is red or green.\n  colour:: red\n- Keeps for months in a cool cellar.\n\t- Store apart from [[Fruit/Pear]]."
    );

    let shown = hover(&mut client, &garden, RECIPES, "[[Fruit/Quince]]");
    assert_eq!(
        shown["contents"]["value"],
        "**Fruit/Quince**\n\nThis page has no file. Go to definition opens `pages/Fruit___Quince.md`.\n\n1 reference"
    );
    let shown = hover(&mut client, &garden, RECIPES, "[[2026-01-03]]");
    let value = shown["contents"]["value"].as_str().unwrap();
    assert!(
        value.starts_with("**Jan 3rd, 2026**\n\n1 reference"),
        "{value}"
    );

    let gone = "((66f1a2b0-0000-4000-8000-00000000dead))";
    assert_eq!(
        hover(&mut client, &garden, RECIPES, gone)["contents"]["value"],
        "No block has this id."
    );
    assert_eq!(
        hover(&mut client, &garden, RECIPES, "Peel four"),
        Value::Null
    );
    assert_eq!(
        hover(&mut client, &garden, RECIPES, "serves::"),
        Value::Null
    );
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn help_describes_definition_references_and_hover() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lsg"))
        .args(["lsp", "help", "section", "behavior"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for phrase in [
        "Go to definition",
        "pages/<file name>.md",
        "Find references",
        "Hover shows",
    ] {
        assert!(help.contains(phrase), "{phrase} is missing from {help}");
    }
}
