//! End-to-end tests of completion in `lsg lsp`: the built binary answers
//! `textDocument/completion` for text typed into a buffer of the fixture garden.

#[path = "lsp/client.rs"]
mod client;

use client::{Client, Garden, Options};
use serde_json::{Value, json};

const DRAFT: &str = "pages/Draft.md";
const APPLE_BLOCK: &str = "66f1a2b0-0000-4000-8000-000000000001";

/// One open buffer, and the client that types into it.
struct Buffer {
    client: Client,
    uri: String,
    version: i64,
}

impl Buffer {
    /// Open a buffer the garden has no file for.
    fn open(garden: &Garden, options: Options) -> Self {
        let mut client = Client::start(garden, options);
        let uri = garden.uri(DRAFT);
        client.open(&uri, "");
        Self {
            client,
            uri,
            version: 1,
        }
    }

    /// Set the buffer to `typed` without its `|`, and ask for completion at the `|`.
    /// Returns the answer and the buffer's text.
    fn complete(&mut self, typed: &str) -> (Value, String) {
        let offset = typed.find('|').expect("the text marks the cursor with |");
        let text = typed.replacen('|', "", 1);
        self.version += 1;
        self.client.notify(
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": self.uri, "version": self.version},
                "contentChanges": [{"text": text}],
            }),
        );
        let answer = self.client.request(
            "textDocument/completion",
            json!({
                "textDocument": {"uri": self.uri},
                "position": client::position_at(&text, offset, self.client.utf8),
            }),
        );
        (answer, text)
    }

    /// The buffer's text after the client accepts the item labelled `label`.
    fn accept(&mut self, typed: &str, label: &str) -> String {
        let (answer, text) = self.complete(typed);
        let edit = &item(&answer, label)["textEdit"];
        let utf8 = self.client.utf8;
        let start = client::offset_of(&text, &edit["range"]["start"], utf8);
        let end = client::offset_of(&text, &edit["range"]["end"], utf8);
        let mut text = text;
        text.replace_range(start..end, edit["newText"].as_str().unwrap());
        text
    }
}

fn labels(answer: &Value) -> Vec<&str> {
    answer["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["label"].as_str().unwrap())
        .collect()
}

fn item<'a>(answer: &'a Value, label: &str) -> &'a Value {
    answer["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["label"] == label)
        .unwrap_or_else(|| panic!("no {label} in {:?}", labels(answer)))
}

#[test]
fn the_server_advertises_completion_and_its_trigger_characters() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());
    assert_eq!(
        client.initialize_result["capabilities"]["completionProvider"]["triggerCharacters"],
        json!(["[", "#", "(", ":"])
    );
    // Wait for the garden to load before leaving.
    client.request("lsg/status", Value::Null);
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn a_page_link_offers_titles_aliases_namespaces_and_journals() {
    let garden = Garden::copy();
    let mut buffer = Buffer::open(&garden, Options::default());

    // Right after the trigger: every name, and the list can grow as the person types.
    let (answer, _) = buffer.complete("- [[|");
    assert_eq!(answer["isIncomplete"], true);
    for name in ["Fruit", "Fruit/Apple", "Recipes", "multi word", "Pomme"] {
        item(&answer, name);
    }

    let (answer, _) = buffer.complete("- See [[Fru|");
    assert_eq!(labels(&answer), ["Fruit", "Fruit/Pear", "Fruit/Apple"]);
    let apple = item(&answer, "Fruit/Apple");
    assert_eq!(apple["textEdit"]["newText"], "Fruit/Apple]]");
    assert_eq!(
        apple["textEdit"]["range"],
        json!({"start": {"line": 0, "character": 8}, "end": {"line": 0, "character": 11}})
    );
    assert_eq!(apple["filterText"], "Fruit/Apple");
    // File.
    assert_eq!(apple["kind"], 17);
    // The items sort in ranked order.
    let sort: Vec<&str> = answer["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["sortText"].as_str().unwrap())
        .collect();
    assert_eq!(sort, ["0000", "0001", "0002"]);

    // An alias names its page and inserts the alias.
    let (answer, _) = buffer.complete("- [[Pom|");
    let alias = item(&answer, "Pomme");
    assert_eq!(alias["labelDetails"]["description"], "Fruit/Apple");
    assert_eq!(alias["detail"], "alias of Fruit/Apple");
    assert_eq!(alias["filterText"], "Pomme");
    assert_eq!(alias["textEdit"]["newText"], "Pomme]]");

    // One namespace level after `/`.
    let (answer, _) = buffer.complete("- [[Fruit/|");
    assert_eq!(labels(&answer)[..2], ["Fruit/Apple", "Fruit/Pear"]);
    assert_eq!(
        buffer.accept("- [[Fruit/pe|", "Fruit/Pear"),
        "- [[Fruit/Pear]]"
    );
    // A namespace page that has no file of its own.
    garden.write("pages/Tools___Editor___Neovim.md", "- An editor\n");
    let uri = garden.uri("pages/Tools___Editor___Neovim.md");
    buffer.client.files_changed(&[(&uri, 1)]);
    let (answer, _) = buffer.complete("- [[Tools/|");
    assert_eq!(labels(&answer)[0], "Tools/Editor");
    // Folder.
    assert_eq!(item(&answer, "Tools/Editor")["kind"], 19);

    // A journal by its title, and by a date in another format, with or without a file.
    let (answer, _) = buffer.complete("- [[jan 2|");
    assert_eq!(labels(&answer)[0], "Jan 2nd, 2026");
    assert_eq!(
        buffer.accept("- On [[2026-01-04|", "Jan 4th, 2026"),
        "- On [[Jan 4th, 2026]]"
    );
    let (answer, _) = buffer.complete("- On [[2026-01-04|");
    let journal = item(&answer, "Jan 4th, 2026");
    assert_eq!(journal["filterText"], "2026-01-04");
    assert_eq!(labels(&answer)[0], "Jan 4th, 2026");

    assert_eq!(buffer.client.shutdown().code(), Some(0));
}

#[test]
fn a_closing_pair_after_the_cursor_is_replaced() {
    let garden = Garden::copy();
    let mut buffer = Buffer::open(&garden, Options::default());
    // `mini.pairs` wrote the `]]` when the person typed `[[`.
    assert_eq!(
        buffer.accept("- See [[Fru|]] today", "Fruit/Apple"),
        "- See [[Fruit/Apple]] today"
    );
    assert_eq!(buffer.accept("- [[|]]", "Recipes"), "- [[Recipes]]");
    assert_eq!(
        buffer.accept("- ((apple red|))", "An apple is red or green."),
        format!("- (({APPLE_BLOCK}))")
    );
    // Inside an embed, with the closing pair and braces already there.
    assert_eq!(
        buffer.accept("- {{embed [[Fru|]]}}", "Fruit/Pear"),
        "- {{embed [[Fruit/Pear]]}}"
    );
    assert_eq!(
        buffer.accept(
            "- {{embed ((pear tart|))}}",
            "Baked the pear tart from [[Recipes]]."
        ),
        "- {{embed ((66f1a2b0-0000-4000-8000-000000000004))}}"
    );
    // The closing pair of a later link stays.
    assert_eq!(
        buffer.accept("- [[Rec| and [[Fruit]]", "Recipes"),
        "- [[Recipes]] and [[Fruit]]"
    );
    assert_eq!(buffer.client.shutdown().code(), Some(0));
}

#[test]
fn a_tag_offers_page_names_and_brackets_a_name_with_a_space() {
    let garden = Garden::copy();
    let mut buffer = Buffer::open(&garden, Options::default());
    let (answer, _) = buffer.complete("- Cake #fru|");
    assert_eq!(labels(&answer), ["Fruit", "Fruit/Pear", "Fruit/Apple"]);
    assert_eq!(item(&answer, "Fruit")["textEdit"]["newText"], "Fruit");
    assert_eq!(
        buffer.accept("- Cake #fru|", "Fruit/Apple"),
        "- Cake #Fruit/Apple"
    );
    assert_eq!(
        buffer.accept("- Cake #mul| and tea", "multi word"),
        "- Cake #[[multi word]] and tea"
    );
    // After `#[[` the name is a page link.
    assert_eq!(
        buffer.accept("- Cake #[[mul|]]", "multi word"),
        "- Cake #[[multi word]]"
    );
    assert_eq!(buffer.client.shutdown().code(), Some(0));
}

#[test]
fn a_block_reference_offers_blocks_by_their_text() {
    let garden = Garden::copy();
    let mut buffer = Buffer::open(&garden, Options::default());
    let (answer, _) = buffer.complete("- ((|");
    // Every id a block writes, each once.
    assert_eq!(answer["items"].as_array().unwrap().len(), 4);
    assert_eq!(answer["isIncomplete"], false);

    let (answer, _) = buffer.complete("- See ((apart PEAR|");
    assert_eq!(labels(&answer), ["Store apart from [[Fruit/Pear]]."]);
    let block = &answer["items"][0];
    assert_eq!(block["labelDetails"]["description"], "Fruit/Apple");
    assert_eq!(block["filterText"], "Store apart from [[Fruit/Pear]].");
    assert_eq!(
        block["textEdit"]["newText"],
        "66f1a2b0-0000-4000-8000-000000000002))"
    );
    assert_eq!(buffer.client.shutdown().code(), Some(0));
}

#[test]
fn a_property_line_offers_keys_and_values() {
    let garden = Garden::copy();
    garden.write(
        "logseq/config.edn",
        &format!(
            "{}\n",
            garden
                .read("logseq/config.edn")
                .trim_end()
                .trim_end_matches('}')
                .to_owned()
                + "\n :block-hidden-properties #{:secret}}"
        ),
    );
    garden.write(
        "pages/Cards.md",
        "- What is a pear? #card\n  card-last-score:: 5\n  secret:: yes\n  source:: a book\n",
    );
    let mut buffer = Buffer::open(&garden, Options::default());

    let (answer, _) = buffer.complete("- Block\n  |");
    let keys = labels(&answer);
    for key in ["type", "alias", "tags", "id", "colour", "serves", "source"] {
        assert!(keys.contains(&key), "{key} is missing from {keys:?}");
    }
    // Logseq's card scheduling properties and `:block-hidden-properties` stay out.
    assert!(!keys.contains(&"card-last-score"), "{keys:?}");
    assert!(!keys.contains(&"secret"), "{keys:?}");
    // The keys written most often come first.
    assert_eq!(keys[..2], ["id", "type"]);
    assert_eq!(answer["isIncomplete"], false);
    // Property.
    assert_eq!(item(&answer, "type")["kind"], 10);

    assert_eq!(
        buffer.accept("- Block\n  ty|", "type"),
        "- Block\n  type:: "
    );
    assert_eq!(buffer.accept("col|", "colour"), "colour:: ");
    // The rest of the key and its `::` are replaced.
    assert_eq!(
        buffer.accept("- Block\n  se|x:: 8", "serves"),
        "- Block\n  serves:: 8"
    );

    // Values, right after the `:` trigger and after a space.
    let (answer, _) = buffer.complete("- Block\n  type::|");
    // Two lines write each value, so they come in name order.
    assert_eq!(labels(&answer), ["[[Category]]", "recipe"]);
    assert_eq!(
        buffer.accept("- Block\n  type::|", "recipe"),
        "- Block\n  type:: recipe"
    );
    assert_eq!(
        buffer.accept("- Block\n  type:: rec|", "recipe"),
        "- Block\n  type:: recipe"
    );
    // Value.
    let (answer, _) = buffer.complete("- Block\n  type:: |");
    assert_eq!(item(&answer, "recipe")["kind"], 12);

    // `tags::` and `alias::` take page names.
    assert_eq!(
        buffer.accept("tags:: sweet, Fru|", "Fruit/Pear"),
        "tags:: sweet, Fruit/Pear"
    );
    assert_eq!(buffer.accept("alias:: rec|", "Recipes"), "alias:: Recipes");

    assert_eq!(buffer.client.shutdown().code(), Some(0));
}

#[test]
fn prose_and_unknown_documents_get_no_items() {
    let garden = Garden::copy();
    let mut buffer = Buffer::open(&garden, Options::default());
    for prose in [
        "- An apple is red|",
        "- [[Fruit]] and|",
        "- # Heading|",
        "- Block\n  some wor|",
    ] {
        let (answer, _) = buffer.complete(prose);
        assert_eq!(answer["items"], json!([]), "{prose}");
        assert_eq!(answer["isIncomplete"], false, "{prose}");
    }
    // A document the server has no text for.
    let answer = buffer.client.request(
        "textDocument/completion",
        json!({
            "textDocument": {"uri": garden.uri("pages/Nowhere.md")},
            "position": {"line": 3, "character": 9},
        }),
    );
    assert_eq!(answer["items"], json!([]));
    assert_eq!(buffer.client.shutdown().code(), Some(0));
}

#[test]
fn a_file_that_is_not_open_completes_from_its_text_on_disk() {
    let garden = Garden::copy();
    garden.write("pages/Half.md", "- Half typed [[Rec\n");
    let mut client = Client::start(&garden, Options::default());
    let uri = garden.uri("pages/Half.md");
    let params = client.at(&uri, "- Half typed [[Rec\n", "[[Rec", 5);
    let answer = client.request("textDocument/completion", params);
    assert_eq!(labels(&answer), ["Recipes"]);
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn text_edits_use_the_negotiated_position_encoding() {
    let garden = Garden::copy();
    for utf8 in [true, false] {
        let options = Options {
            utf8,
            ..Options::default()
        };
        let mut buffer = Buffer::open(&garden, options);
        let (answer, _) = buffer.complete("- Café 🍎 [[Fru|]]");
        // `é` is two UTF-8 units and one UTF-16 unit; the apple is four and two.
        let (start, end) = if utf8 { (15, 20) } else { (12, 17) };
        assert_eq!(
            item(&answer, "Fruit")["textEdit"]["range"],
            json!({
                "start": {"line": 0, "character": start},
                "end": {"line": 0, "character": end},
            })
        );
        assert_eq!(
            buffer.accept("- Café 🍎 [[Fru|]] é", "Fruit"),
            "- Café 🍎 [[Fruit]] é"
        );
        assert_eq!(buffer.client.shutdown().code(), Some(0));
    }
}

#[test]
fn a_long_list_is_cut_and_marked_incomplete() {
    let garden = Garden::copy();
    for n in 0..150 {
        garden.write(
            &format!("pages/Note {n:03}.md"),
            &format!("- Note\n  key{n:03}:: value\n"),
        );
    }
    let mut buffer = Buffer::open(&garden, Options::default());
    let (answer, _) = buffer.complete("- [[note|");
    assert_eq!(answer["items"].as_array().unwrap().len(), 100);
    assert_eq!(answer["isIncomplete"], true);
    assert_eq!(labels(&answer)[0], "Note 000");
    // More typing reaches a page the cut list left out.
    let (answer, _) = buffer.complete("- [[note 149|");
    assert_eq!(labels(&answer)[0], "Note 149");

    let (answer, _) = buffer.complete("- Block\n  key|");
    assert_eq!(answer["items"].as_array().unwrap().len(), 100);
    assert_eq!(answer["isIncomplete"], true);
    let (answer, _) = buffer.complete("- Block\n  key14|");
    assert_eq!(answer["isIncomplete"], false);
    item(&answer, "key149");
    assert_eq!(buffer.client.shutdown().code(), Some(0));
}
