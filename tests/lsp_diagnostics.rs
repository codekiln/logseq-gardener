//! End-to-end tests of the diagnostics `lsg lsp` publishes: broken block references,
//! block ids that more than one block writes, and embeds of pages without a file.

#[path = "lsp/client.rs"]
mod client;

use client::{Client, Garden, Options, position_at};
use serde_json::{Value, json};

const APPLE: &str = "pages/Fruit___Apple.md";
const PEAR: &str = "pages/Fruit___Pear.md";
const RECIPES: &str = "pages/Recipes.md";
const JOURNAL: &str = "journals/2026_01_03.md";

/// A block id no block of the fixture writes; `pages/Recipes.md` refers to it.
const DEAD: &str = "66f1a2b0-0000-4000-8000-00000000dead";
/// The id the fixture writes in `journals/2026_01_03.md` and in `pages/Fruit___Pear.md`.
const SHARED: &str = "66f1a2b0-0000-4000-8000-000000000003";
/// The id of a block in `pages/Fruit___Apple.md`.
const APPLE_BLOCK: &str = "66f1a2b0-0000-4000-8000-000000000001";
/// An id no block of the fixture writes and nothing refers to.
const FRESH: &str = "66f1a2b0-0000-4000-8000-000000000009";

/// Every list of diagnostics the server has published for `uri` since the last call, oldest
/// first. The server handles messages in order, so every list that an earlier notification
/// caused has arrived once it answers a request.
fn published(client: &mut Client, uri: &str) -> Vec<Value> {
    client.request("lsg/status", Value::Null);
    let mut lists = Vec::new();
    client.notifications.retain(|notification| {
        let ours = notification["method"] == "textDocument/publishDiagnostics"
            && notification["params"]["uri"] == uri;
        if ours {
            lists.push(notification["params"]["diagnostics"].clone());
        }
        !ours
    });
    lists
}

/// The one list of diagnostics the server published for `uri` since the last call.
fn diagnostics(client: &mut Client, uri: &str) -> Vec<Value> {
    let mut lists = published(client, uri);
    assert_eq!(lists.len(), 1, "lists published for {uri}: {lists:?}");
    lists.remove(0).as_array().unwrap().clone()
}

/// The protocol range of the first `needle` in `text`.
fn range_of(client: &Client, text: &str, needle: &str) -> Value {
    let start = text
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} is not in the text"));
    json!({
        "start": position_at(text, start, client.utf8),
        "end": position_at(text, start + needle.len(), client.utf8),
    })
}

/// Open a document of the garden with the text of its file. Returns its URI and text.
fn open(client: &mut Client, garden: &Garden, path: &str) -> (String, String) {
    let (uri, text) = (garden.uri(path), garden.read(path));
    client.open(&uri, &text);
    (uri, text)
}

#[test]
fn an_open_page_shows_each_problem_and_an_edit_clears_it() {
    let garden = Garden::copy();
    garden.write(
        "pages/Embeds.md",
        "- {{embed [[Fruit/Quince]]}} and {{embed ((66f1a2b0-0000-4000-8000-00000000dead))}}\n",
    );
    let mut client = Client::start(&garden, Options::default());

    // A block reference whose block is gone.
    let (recipes, text) = open(&mut client, &garden, RECIPES);
    let reference = format!("(({DEAD}))");
    assert_eq!(
        diagnostics(&mut client, &recipes),
        [json!({
            "range": range_of(&client, &text, &reference),
            "severity": 1,
            "code": "missing-block",
            "source": "lsg",
            "message": format!("No block has the id {DEAD}"),
        })]
    );
    client.replace(&recipes, 2, &text, DEAD, APPLE_BLOCK);
    assert_eq!(diagnostics(&mut client, &recipes), [] as [Value; 0]);

    // The same id on a block of this page and a block of a journal.
    let (pear, text) = open(&mut client, &garden, PEAR);
    let journal_text = garden.read(JOURNAL);
    assert_eq!(
        diagnostics(&mut client, &pear),
        [json!({
            "range": range_of(&client, &text, SHARED),
            "severity": 1,
            "code": "duplicate-block-id",
            "source": "lsg",
            "message": format!("More than one block has the id {SHARED}"),
            "relatedInformation": [{
                "location": {
                    "uri": garden.uri(JOURNAL),
                    "range": range_of(&client, &journal_text, SHARED),
                },
                "message": "Another block with this id",
            }],
        })]
    );
    client.replace(&pear, 2, &text, SHARED, FRESH);
    assert_eq!(diagnostics(&mut client, &pear), [] as [Value; 0]);

    // An embed of a page without a file, and an embed of a block that is gone.
    let (embeds, text) = open(&mut client, &garden, "pages/Embeds.md");
    let found = diagnostics(&mut client, &embeds);
    assert_eq!(
        found,
        [
            json!({
                "range": range_of(&client, &text, "{{embed [[Fruit/Quince]]}}"),
                "severity": 2,
                "code": "missing-embedded-page",
                "source": "lsg",
                "message": "The embedded page Fruit/Quince has no file",
            }),
            json!({
                "range": range_of(&client, &text, &format!("{{{{embed (({DEAD}))}}}}")),
                "severity": 1,
                "code": "missing-block",
                "source": "lsg",
                "message": format!("No block has the id {DEAD}"),
            }),
        ]
    );
    // An alias of a page with a file.
    let text = client.replace(&embeds, 2, &text, "Fruit/Quince", "Pyrus");
    assert_eq!(diagnostics(&mut client, &embeds).len(), 1);
    client.replace(&embeds, 3, &text, DEAD, APPLE_BLOCK);
    assert_eq!(diagnostics(&mut client, &embeds), [] as [Value; 0]);

    // No edit touched the other open documents' problems, so nothing more was sent.
    assert_eq!(published(&mut client, &recipes), [] as [Value; 0]);
    assert_eq!(published(&mut client, &pear), [] as [Value; 0]);
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn a_link_to_a_page_without_a_file_gets_no_diagnostic() {
    let garden = Garden::copy();
    garden.write(
        "pages/Plans.md",
        "tags:: Orchard, [[Someday]]\nalias:: Ideas\n\n- Plant [[Fruit/Quince]] and [a medlar]([[Medlar]]) #orchard #[[two words]]\n- On [[Mar 4th, 2031]] ((not a block id))\n",
    );
    let mut client = Client::start(&garden, Options::default());

    let (plans, _) = open(&mut client, &garden, "pages/Plans.md");
    assert_eq!(diagnostics(&mut client, &plans), [] as [Value; 0]);

    // `pages/Recipes.md` links to `[[Fruit/Quince]]`, which has no file; its one problem is
    // the block reference.
    let (recipes, _) = open(&mut client, &garden, RECIPES);
    let found = diagnostics(&mut client, &recipes);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["code"], "missing-block");
}

#[test]
fn a_fix_in_another_file_clears_the_diagnostic() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());
    let (recipes, _) = open(&mut client, &garden, RECIPES);
    assert_eq!(diagnostics(&mut client, &recipes).len(), 1);
    let (apple, apple_text) = open(&mut client, &garden, APPLE);
    assert_eq!(diagnostics(&mut client, &apple), [] as [Value; 0]);

    // Another open document gives a block the missing id.
    let apple_text = client.replace(&apple, 2, &apple_text, APPLE_BLOCK, DEAD);
    assert_eq!(diagnostics(&mut client, &recipes), [] as [Value; 0]);
    // `pages/Fruit___Pear.md` refers to the id the block had, and is not open.
    assert_eq!(published(&mut client, &garden.uri(PEAR)), [] as [Value; 0]);

    // The id goes away again, and the reference is broken again.
    client.replace(&apple, 3, &apple_text, DEAD, APPLE_BLOCK);
    let found = diagnostics(&mut client, &recipes);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["code"], "missing-block");
    assert_eq!(published(&mut client, &apple), [] as [Value; 0]);

    // Two open documents share an id; changing it in one clears both.
    let (pear, _) = open(&mut client, &garden, PEAR);
    let (journal, journal_text) = open(&mut client, &garden, JOURNAL);
    assert_eq!(
        diagnostics(&mut client, &pear)[0]["code"],
        "duplicate-block-id"
    );
    let found = diagnostics(&mut client, &journal);
    assert_eq!(found[0]["code"], "duplicate-block-id");
    assert_eq!(found[0]["relatedInformation"][0]["location"]["uri"], pear);
    client.replace(&journal, 2, &journal_text, SHARED, FRESH);
    assert_eq!(diagnostics(&mut client, &journal), [] as [Value; 0]);
    assert_eq!(diagnostics(&mut client, &pear), [] as [Value; 0]);
}

#[test]
fn files_changed_outside_the_editor_update_the_diagnostics() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());
    let (pear, _) = open(&mut client, &garden, PEAR);
    assert_eq!(diagnostics(&mut client, &pear).len(), 1);

    // Another program gives the journal's block a new id, which fixes the open page.
    let journal = garden.uri(JOURNAL);
    garden.write(JOURNAL, &garden.read(JOURNAL).replace(SHARED, FRESH));
    client.files_changed(&[(&journal, 2)]);
    assert_eq!(diagnostics(&mut client, &pear), [] as [Value; 0]);
    // The journal is not open and has no problem, so the server sent nothing for it.
    assert_eq!(published(&mut client, &journal), [] as [Value; 0]);

    // Another program writes a broken reference into a page that is not open.
    let fruit = garden.uri("pages/Fruit.md");
    let text = format!("{}- (({DEAD}))\n", garden.read("pages/Fruit.md"));
    garden.write("pages/Fruit.md", &text);
    client.files_changed(&[(&fruit, 2)]);
    let found = diagnostics(&mut client, &fruit);
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0]["range"],
        range_of(&client, &text, &format!("(({DEAD}))"))
    );

    // A fix in an open document reaches the page that is not open.
    let (apple, apple_text) = open(&mut client, &garden, APPLE);
    assert_eq!(diagnostics(&mut client, &apple), [] as [Value; 0]);
    let apple_text = client.replace(&apple, 2, &apple_text, "colour:: red", "colour:: green");
    assert_eq!(published(&mut client, &fruit), [] as [Value; 0]);
    let apple_text = client.replace(&apple, 3, &apple_text, APPLE_BLOCK, DEAD);
    assert_eq!(diagnostics(&mut client, &fruit), [] as [Value; 0]);
    // The server follows a document that is not open only while it has a published
    // problem, so the page is reported again when its file next changes.
    client.replace(&apple, 4, &apple_text, DEAD, APPLE_BLOCK);
    assert_eq!(published(&mut client, &fruit), [] as [Value; 0]);
    client.files_changed(&[(&fruit, 2)]);
    assert_eq!(diagnostics(&mut client, &fruit).len(), 1);

    // Deleting the file clears its diagnostics.
    garden.remove("pages/Fruit.md");
    client.files_changed(&[(&fruit, 3)]);
    assert_eq!(diagnostics(&mut client, &fruit), [] as [Value; 0]);
    client.files_changed(&[(&fruit, 3)]);
    assert_eq!(published(&mut client, &fruit), [] as [Value; 0]);
}

#[test]
fn closing_a_document_clears_its_diagnostics() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());
    let (recipes, text) = open(&mut client, &garden, RECIPES);
    assert_eq!(diagnostics(&mut client, &recipes).len(), 1);
    client.close(&recipes);
    assert_eq!(diagnostics(&mut client, &recipes), [] as [Value; 0]);

    // Opening it again publishes again.
    client.open(&recipes, &text);
    assert_eq!(diagnostics(&mut client, &recipes).len(), 1);

    // Closing a document drops its unsaved text, and with it what the text fixed elsewhere.
    let (apple, apple_text) = open(&mut client, &garden, APPLE);
    assert_eq!(diagnostics(&mut client, &apple), [] as [Value; 0]);
    client.replace(&apple, 2, &apple_text, APPLE_BLOCK, DEAD);
    assert_eq!(diagnostics(&mut client, &recipes), [] as [Value; 0]);
    client.close(&apple);
    assert_eq!(diagnostics(&mut client, &recipes).len(), 1);
    // The closed document had no problem, so closing it sent nothing for it.
    assert_eq!(published(&mut client, &apple), [] as [Value; 0]);
}

#[test]
fn a_client_without_related_information_reads_the_places_in_the_message() {
    let garden = Garden::copy();
    let options = Options {
        related_information: false,
        ..Options::default()
    };
    let mut client = Client::start(&garden, options);
    let (pear, _) = open(&mut client, &garden, PEAR);
    let found = diagnostics(&mut client, &pear);
    assert_eq!(found.len(), 1);
    assert!(found[0].get("relatedInformation").is_none());
    assert_eq!(
        found[0]["message"],
        format!("More than one block has the id {SHARED}. Also at {JOURNAL}:2")
    );
}

#[test]
fn ranges_use_the_negotiated_encoding() {
    for utf8 in [true, false] {
        let garden = Garden::copy();
        let text = format!("- Café 🍎 (({DEAD}))\n");
        garden.write("pages/Notes.md", &text);
        let options = Options {
            utf8,
            ..Options::default()
        };
        let mut client = Client::start(&garden, options);
        let (notes, _) = open(&mut client, &garden, "pages/Notes.md");
        let found = diagnostics(&mut client, &notes);
        // `- Café 🍎 ` is 13 UTF-8 units and 10 UTF-16 units.
        let start = if utf8 { 13 } else { 10 };
        assert_eq!(
            found[0]["range"],
            json!({
                "start": {"line": 0, "character": start},
                "end": {"line": 0, "character": start + 40},
            })
        );
        assert_eq!(
            found[0]["range"],
            range_of(&client, &text, &format!("(({DEAD}))"))
        );
    }
}
