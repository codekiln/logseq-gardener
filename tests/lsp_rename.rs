//! End-to-end tests of renaming a page in `lsg lsp`.

#[path = "lsp/client.rs"]
mod client;

use std::path::{Path, PathBuf};

use client::{Client, Garden, Options};
use serde_json::{Value, json};

const RECIPES: &str = "pages/Recipes.md";
const FRUIT: &str = "pages/Fruit.md";
const APPLE: &str = "pages/Fruit___Apple.md";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/lsp-rename-garden")
        .join(name)
}

fn before() -> Garden {
    Garden::copy_of(&fixture("before"))
}

/// The params of a rename at the first `needle` of a file, three characters in.
fn rename_at(client: &Client, garden: &Garden, file: &str, needle: &str, name: &str) -> Value {
    let mut params = client.at(&garden.uri(file), &garden.read(file), needle, 3);
    params["newName"] = json!(name);
    params
}

fn rename(
    client: &mut Client,
    garden: &Garden,
    file: &str,
    needle: &str,
    name: &str,
) -> Result<Value, Value> {
    let params = rename_at(client, garden, file, needle, name);
    client.request_raw("textDocument/rename", params)
}

/// The answer to `textDocument/prepareRename` at the first `needle` of a file, as the text
/// of its range and its placeholder.
fn prepare(client: &mut Client, garden: &Garden, file: &str, needle: &str) -> (String, String) {
    let text = garden.read(file);
    let params = client.at(&garden.uri(file), &text, needle, 3);
    let answer = client.request("textDocument/prepareRename", params);
    let start = client::offset_of(&text, &answer["range"]["start"], client.utf8);
    let end = client::offset_of(&text, &answer["range"]["end"], client.utf8);
    (
        text[start..end].to_owned(),
        answer["placeholder"].as_str().unwrap().to_owned(),
    )
}

fn relative(garden: &Garden, uri: &Value) -> String {
    client::uri_to_path(uri.as_str().unwrap())
        .strip_prefix(&garden.root)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}

/// Start a server for a client that advertises the given `workspace.workspaceEdit`.
fn start_with(garden: &Garden, workspace_edit: Value) -> Client {
    let mut client = Client::spawn(true);
    let uri = garden.uri("");
    let uri = uri.trim_end_matches('/');
    client
        .request_raw(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": uri,
                "workspaceFolders": [{"uri": uri, "name": "garden"}],
                "capabilities": {
                    "general": {"positionEncodings": ["utf-8"]},
                    "workspace": {
                        "didChangeWatchedFiles": {"dynamicRegistration": true},
                        "workspaceEdit": workspace_edit,
                    },
                },
            }),
        )
        .unwrap();
    client.notify("initialized", json!({}));
    client
}

#[test]
fn a_rename_rewrites_the_garden_as_the_expected_copy() {
    for utf8 in [true, false] {
        let garden = before();
        let options = Options {
            utf8,
            ..Options::default()
        };
        let mut client = Client::start(&garden, options);
        assert_eq!(
            client.initialize_result["capabilities"]["renameProvider"],
            json!({"prepareProvider": true})
        );
        // An open buffer gets an edit for the version the server holds.
        client.open(&garden.uri(RECIPES), &garden.read(RECIPES));
        assert_eq!(
            prepare(&mut client, &garden, RECIPES, "[[Fruit]] dishes"),
            ("Fruit".to_owned(), "Fruit".to_owned())
        );
        let edit = rename(
            &mut client,
            &garden,
            RECIPES,
            "[[Fruit]] dishes",
            " Sweet Produce ",
        )
        .unwrap();

        let changes = edit["documentChanges"].as_array().unwrap();
        let mut edited = Vec::new();
        for change in changes {
            if change["kind"] == "rename" {
                assert_eq!(change.get("options"), None);
            } else {
                let file = relative(&garden, &change["textDocument"]["uri"]);
                let version = &change["textDocument"]["version"];
                if file == RECIPES {
                    assert_eq!(version, &json!(1));
                } else {
                    assert!(version.is_null(), "{file} has version {version}");
                }
                edited.push(file);
            }
        }
        let renames: Vec<(String, String)> = changes
            .iter()
            .filter(|change| change["kind"] == "rename")
            .map(|change| {
                (
                    relative(&garden, &change["oldUri"]),
                    relative(&garden, &change["newUri"]),
                )
            })
            .collect();
        let pair = |from: &str, to: &str| (from.to_owned(), to.to_owned());
        assert_eq!(
            renames,
            [
                pair(FRUIT, "pages/Sweet Produce.md"),
                pair(APPLE, "pages/Sweet Produce___Apple.md"),
                pair(
                    "pages/Fruit___Apple___Cox.md",
                    "pages/Sweet Produce___Apple___Cox.md"
                ),
            ]
        );
        for file in [FRUIT, APPLE, "pages/Fruit___Apple___Cox.md"] {
            let edit_at = changes.iter().position(|change| {
                change.get("kind").is_none()
                    && relative(&garden, &change["textDocument"]["uri"]) == file
            });
            let rename_at = changes.iter().position(|change| {
                change["kind"] == "rename" && relative(&garden, &change["oldUri"]) == file
            });
            assert!(edit_at < rename_at, "{file}: {edit_at:?} {rename_at:?}");
        }
        assert!(!edited.contains(&"pages/Untouched.md".to_owned()));
        assert!(!edited.contains(&"pages/Fruitcake.md".to_owned()));

        client.apply_edit(&edit);
        garden.assert_same_as(&fixture("after"));
        assert!(client.shutdown().success());
    }
}

#[test]
fn prepare_rename_names_the_page_under_the_cursor() {
    let garden = before();
    let mut client = Client::start(&garden, Options::default());
    let page = |range: &str, name: &str| (range.to_owned(), name.to_owned());
    // An alias leads to its page, and a tag and a namespace child are pages.
    assert_eq!(
        prepare(&mut client, &garden, RECIPES, "[[Produce]]"),
        page("Produce", "Fruit")
    );
    assert_eq!(
        prepare(&mut client, &garden, RECIPES, "#Fruit/Apple"),
        page("Fruit/Apple", "Fruit/Apple")
    );
    assert_eq!(
        prepare(&mut client, &garden, APPLE, "Fruit, sweet"),
        page("Fruit", "Fruit")
    );
    // A page with no file is named as written, and ordinary text names its own page.
    assert_eq!(
        prepare(&mut client, &garden, FRUIT, "[[Food]]"),
        page("Food", "Food")
    );
    assert_eq!(
        prepare(&mut client, &garden, APPLE, "An apple"),
        page("", "Fruit/Apple")
    );
    // A block reference has no name.
    let text = garden.read(RECIPES);
    let params = client.at(&garden.uri(RECIPES), &text, "((66f1", 4);
    let error = client
        .request_raw("textDocument/prepareRename", params)
        .unwrap_err();
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("block reference"),
        "{error}"
    );
    assert!(client.shutdown().success());
}

#[test]
fn a_refused_rename_is_an_error_and_changes_nothing() {
    let garden = before();
    let mut client = Client::start(&garden, Options::default());
    let message = |answer: Result<Value, Value>| {
        let error = answer.unwrap_err();
        assert_eq!(error["code"], json!(-32803), "{error}");
        error["message"].as_str().unwrap().to_owned()
    };
    let taken = message(rename(
        &mut client,
        &garden,
        RECIPES,
        "[[Fruit]] dishes",
        "fruitcake",
    ));
    assert!(
        taken.contains("already belongs to the page in pages/Fruitcake.md"),
        "{taken}"
    );
    // By an alias of another page, and for a namespace child.
    let alias = message(rename(
        &mut client,
        &garden,
        RECIPES,
        "[[Fruitcake]]",
        "malus",
    ));
    assert!(alias.contains("pages/Fruit___Apple.md"), "{alias}");
    garden.write("pages/Veg___Apple.md", "- taken\n");
    client.files_changed(&[(&garden.uri("pages/Veg___Apple.md"), 1)]);
    let child = message(rename(
        &mut client,
        &garden,
        RECIPES,
        "[[Fruit]] dishes",
        "Veg",
    ));
    assert!(child.contains("Veg/Apple"), "{child}");
    garden.remove("pages/Veg___Apple.md");
    let empty = message(rename(
        &mut client,
        &garden,
        RECIPES,
        "[[Fruit]] dishes",
        "  ",
    ));
    assert!(empty.contains("empty"), "{empty}");
    let date = message(rename(
        &mut client,
        &garden,
        RECIPES,
        "[[Fruit]] dishes",
        "2026-02-03",
    ));
    assert!(date.contains("journal date"), "{date}");
    garden.assert_same_as(&fixture("before"));
    assert!(client.shutdown().success());
}

#[test]
fn a_rename_of_letter_case_only_reaches_the_new_file_name() {
    let garden = before();
    let mut client = Client::start(&garden, Options::default());
    let edit = rename(&mut client, &garden, FRUIT, "Fruit grows", "FRUIT").unwrap();
    client.apply_edit(&edit);
    let mut names: Vec<String> = std::fs::read_dir(garden.path("pages"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "FRUIT.md",
            "FRUIT___Apple.md",
            "FRUIT___Apple___Cox.md",
            "Fruitcake.md",
            "Recipes.md",
            "Untouched.md",
        ]
    );
    assert_eq!(
        garden.read("journals/2026_01_02.md"),
        "- Bought [[FRUIT]] at the market.\n\t- TODO more #FRUIT/Apple/Cox\n"
    );
    assert!(garden.read("logseq/config.edn").contains(":page \"FRUIT\""));
    assert!(client.shutdown().success());
}

#[test]
fn a_journal_keeps_its_file_name() {
    let garden = before();
    garden.write(
        "pages/Diary.md",
        "- See [[Jan 2nd, 2026]] and [[2026-01-02]].\n",
    );
    let mut client = Client::start(&garden, Options::default());
    let edit = rename(
        &mut client,
        &garden,
        "pages/Diary.md",
        "[[Jan 2nd",
        "Market Day",
    )
    .unwrap();
    assert!(
        edit["documentChanges"]
            .as_array()
            .unwrap()
            .iter()
            .all(|change| change.get("kind").is_none()),
        "{edit}"
    );
    client.apply_edit(&edit);
    assert_eq!(
        garden.read("journals/2026_01_02.md"),
        "title:: Market Day\n\n- Bought [[Fruit]] at the market.\n\t- TODO more #fruit/Apple/Cox\n"
    );
    assert_eq!(
        garden.read("pages/Diary.md"),
        "- See [[Market Day]] and [[2026-01-02]].\n"
    );
    assert!(client.shutdown().success());
}

#[test]
fn an_unsaved_buffer_is_renamed_as_the_editor_shows_it() {
    let garden = before();
    let mut client = Client::start(&garden, Options::default());
    let uri = garden.uri("pages/Untouched.md");
    let text = garden.read("pages/Untouched.md");
    client.open(&uri, &text);
    let text = client.replace(&uri, 5, &text, "deeper", "deeper é [[fruit]]");
    let params = client.at(&uri, &text, "[[fruit]]", 3);
    let mut params = params;
    params["newName"] = json!("Crop");
    let edit = client.request("textDocument/rename", params);
    let change = edit["documentChanges"]
        .as_array()
        .unwrap()
        .iter()
        .find(|change| change["textDocument"]["uri"] == json!(uri))
        .unwrap_or_else(|| panic!("{edit}"));
    assert_eq!(change["textDocument"]["version"], json!(5));
    let range = &change["edits"][0]["range"];
    let start = client::offset_of(&text, &range["start"], true);
    let end = client::offset_of(&text, &range["end"], true);
    assert_eq!(&text[start..end], "fruit");
    assert_eq!(change["edits"][0]["newText"], json!("Crop"));
    assert!(client.shutdown().success());
}

#[test]
fn a_file_rename_needs_a_client_that_renames_files() {
    let garden = before();
    for workspace_edit in [
        json!({"documentChanges": true, "resourceOperations": ["create"]}),
        json!({"documentChanges": true}),
        json!({}),
    ] {
        let mut client = start_with(&garden, workspace_edit.clone());
        let error = rename(&mut client, &garden, RECIPES, "[[Fruit]] dishes", "Crop").unwrap_err();
        let message = error["message"].as_str().unwrap();
        assert!(
            message.contains("workspace.workspaceEdit.resourceOperations"),
            "{message}"
        );
        // A page with no file needs no file rename.
        let edit = rename(&mut client, &garden, FRUIT, "[[Food]]", "Meal").unwrap();
        let edits = if workspace_edit == json!({}) {
            edit["changes"][garden.uri(FRUIT)].clone()
        } else {
            edit["documentChanges"][0]["edits"].clone()
        };
        assert_eq!(edits[0]["newText"], json!("Meal"), "{edit}");
        assert!(client.shutdown().success());
    }
    garden.assert_same_as(&fixture("before"));

    // Neovim lists the resource operations and leaves `documentChanges` out.
    let mut client = start_with(&garden, json!({"resourceOperations": ["rename"]}));
    let edit = rename(
        &mut client,
        &garden,
        RECIPES,
        "[[Fruit]] dishes",
        "Sweet Produce",
    )
    .unwrap();
    client.apply_edit(&edit);
    garden.assert_same_as(&fixture("after"));
    assert!(client.shutdown().success());
}

#[test]
fn help_describes_rename() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lsg"))
        .args(["lsp", "help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for phrase in [
        "Rename on a page link",
        "renames its namespace children",
        "Links written with an alias stay",
        "belongs to another page",
        "rename resource operation",
    ] {
        assert!(help.contains(phrase), "{phrase} is missing from {help}");
    }
}

/// The last list of diagnostics the server published for `uri`, once it has handled every
/// message sent so far.
fn last_diagnostics(client: &mut Client, uri: &str) -> Option<Value> {
    client.request("lsg/status", Value::Null);
    let mut last = None;
    client.notifications.retain(|notification| {
        let ours = notification["method"] == "textDocument/publishDiagnostics"
            && notification["params"]["uri"] == uri;
        if ours {
            last = Some(notification["params"]["diagnostics"].clone());
        }
        !ours
    });
    last
}

#[test]
fn diagnostics_follow_the_edits_and_file_events_of_a_rename() {
    let garden = before();
    let mut client = Client::start(&garden, Options::default());
    let recipes = garden.uri(RECIPES);
    client.open(&recipes, &garden.read(RECIPES));
    assert_eq!(last_diagnostics(&mut client, &recipes), Some(json!([])));

    let edit = rename(
        &mut client,
        &garden,
        RECIPES,
        "[[Fruit]] dishes",
        "Sweet Produce",
    )
    .unwrap();
    client.apply_edit(&edit);
    // The editor reports the buffer's new text ahead of the renamed files, so the embed
    // names a page with no file for a moment.
    client.notify(
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": recipes, "version": 2},
            "contentChanges": [{"text": garden.read(RECIPES)}],
        }),
    );
    let between = last_diagnostics(&mut client, &recipes).unwrap();
    assert_eq!(
        between[0]["code"],
        json!("missing-embedded-page"),
        "{between}"
    );

    let mut events = Vec::new();
    for change in edit["documentChanges"].as_array().unwrap() {
        if change["kind"] == "rename" {
            events.push((change["oldUri"].as_str().unwrap().to_owned(), 3));
            events.push((change["newUri"].as_str().unwrap().to_owned(), 1));
        }
    }
    let events: Vec<(&str, u8)> = events
        .iter()
        .map(|(uri, kind)| (uri.as_str(), *kind))
        .collect();
    client.files_changed(&events);
    assert_eq!(last_diagnostics(&mut client, &recipes), Some(json!([])));
    assert!(client.shutdown().success());
}
