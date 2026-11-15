//! End-to-end tests of renaming a property key in `lsg lsp`.

#[path = "lsp/client.rs"]
mod client;

use std::path::{Path, PathBuf};

use client::{Client, Garden, Options};
use serde_json::{Value, json};

const BOOK: &str = "pages/The Dispossessed.md";
const FILMS: &str = "pages/Films.md";
const JOURNAL: &str = "journals/2026_01_02.md";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/lsp-rename-property-garden")
        .join(name)
}

fn before() -> Garden {
    Garden::copy_of(&fixture("before"))
}

/// Rename at the first `needle` of a file, one character in.
fn rename(
    client: &mut Client,
    garden: &Garden,
    file: &str,
    needle: &str,
    name: &str,
) -> Result<Value, Value> {
    let mut params = client.at(&garden.uri(file), &garden.read(file), needle, 1);
    params["newName"] = json!(name);
    client.request_raw("textDocument/rename", params)
}

/// The message of a refused request, which carries the code for a failed request.
fn message(answer: Result<Value, Value>) -> String {
    let error = answer.unwrap_err();
    assert_eq!(error["code"], json!(-32803), "{error}");
    error["message"].as_str().unwrap().to_owned()
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
fn a_rename_rewrites_the_key_in_every_file_as_the_expected_copy() {
    for utf8 in [true, false] {
        let garden = before();
        let options = Options {
            utf8,
            ..Options::default()
        };
        let mut client = Client::start(&garden, options);
        // An open buffer gets an edit for the version the server holds.
        client.open(&garden.uri(FILMS), &garden.read(FILMS));
        let edit = rename(
            &mut client,
            &garden,
            FILMS,
            "type::   documentary",
            " kind ",
        )
        .unwrap();

        let changes = edit["documentChanges"].as_array().unwrap();
        let mut edited = Vec::new();
        let mut edits = 0;
        for change in changes {
            assert_eq!(change.get("kind"), None, "{change}");
            let file = relative(&garden, &change["textDocument"]["uri"]);
            let version = &change["textDocument"]["version"];
            if file == FILMS {
                assert_eq!(version, &json!(1));
            } else {
                assert!(version.is_null(), "{file} has version {version}");
            }
            for edit in change["edits"].as_array().unwrap() {
                assert_eq!(edit["newText"], json!("kind"));
                edits += 1;
            }
            edited.push(file);
        }
        assert_eq!(edited, [JOURNAL, FILMS, BOOK]);
        assert_eq!(edits, 7);

        client.apply_edit(&edit);
        garden.assert_same_as(&fixture("after"));
        assert!(client.shutdown().success());
    }
}

#[test]
fn prepare_rename_names_the_key_as_written() {
    let garden = before();
    let mut client = Client::start(&garden, Options::default());
    let text = garden.read(BOOK);
    for (needle, inside) in [("Type:: book", 0), ("Type:: book", 2), ("Type:: book", 4)] {
        let params = client.at(&garden.uri(BOOK), &text, needle, inside);
        let answer = client.request("textDocument/prepareRename", params);
        let start = client::offset_of(&text, &answer["range"]["start"], true);
        let end = client::offset_of(&text, &answer["range"]["end"], true);
        assert_eq!(&text[start..end], "Type", "{inside}");
        assert_eq!(answer["placeholder"], json!("Type"));
    }
    // A key with a built-in meaning is refused before the editor asks for a name.
    for needle in ["tags:: type", "collapsed:: true"] {
        let params = client.at(&garden.uri(BOOK), &text, needle, 1);
        let refused = message(client.request_raw("textDocument/prepareRename", params));
        let key = needle.split("::").next().unwrap();
        assert_eq!(
            refused,
            format!("cannot rename: the property key {key} has a built-in meaning in Logseq")
        );
    }
    assert!(client.shutdown().success());
}

#[test]
fn a_refused_rename_is_an_error_and_changes_nothing() {
    let garden = before();
    let mut client = Client::start(&garden, Options::default());
    // From a reserved key, and to one in another spelling.
    let from = message(rename(&mut client, &garden, BOOK, "collapsed::", "folded"));
    assert!(
        from.contains("the property key collapsed has a built-in meaning"),
        "{from}"
    );
    for reserved in ["tags", "ID", "custom_id", "title::", "logseq.color"] {
        let to = message(rename(&mut client, &garden, BOOK, "Type::", reserved));
        assert!(to.contains("has a built-in meaning in Logseq"), "{to}");
    }
    // A key Logseq would not read.
    for invalid in ["", "my key", "a:b", "#kind", "kind(x)"] {
        let refused = message(rename(&mut client, &garden, BOOK, "Type::", invalid));
        assert!(
            refused.contains("the new key is empty") || refused.contains("would not read"),
            "{refused}"
        );
    }
    // A block that writes both keys would lose a value in Logseq.
    let both = message(rename(&mut client, &garden, FILMS, "type:: film", "rating"));
    assert_eq!(
        both,
        "cannot rename: the block at line 1 of pages/Films.md writes both type and rating, and Logseq reads one value for a key on a block; remove one of the two lines first"
    );
    garden.assert_same_as(&fixture("before"));
    assert!(client.shutdown().success());
}

#[test]
fn a_key_in_use_on_other_blocks_takes_the_renamed_lines() {
    let garden = before();
    let mut client = Client::start(&garden, Options::default());
    // `due_date` and `due-date` are one key in Logseq, and `status` is written elsewhere.
    garden.write("pages/Extra.md", "Due-Date:: [[Jan 3rd, 2026]]\n");
    client.files_changed(&[(&garden.uri("pages/Extra.md"), 1)]);
    let edit = rename(&mut client, &garden, FILMS, "due_date::", "status").unwrap();
    client.apply_edit(&edit);
    assert_eq!(
        garden.read("pages/Extra.md"),
        "status:: [[Jan 3rd, 2026]]\n"
    );
    assert!(
        garden
            .read(FILMS)
            .contains("\tstatus:: [[Jan 2nd, 2026]]\n")
    );
    assert_eq!(
        garden.read("pages/Untouched.md"),
        std::fs::read_to_string(fixture("before").join("pages/Untouched.md")).unwrap()
    );
    assert!(client.shutdown().success());
}

#[test]
fn a_client_without_resource_operations_gets_the_text_edits() {
    for workspace_edit in [json!({}), json!({"documentChanges": true})] {
        let garden = before();
        let mut client = start_with(&garden, workspace_edit.clone());
        let edit = rename(&mut client, &garden, JOURNAL, "type:: reading", "kind").unwrap();
        if workspace_edit == json!({}) {
            assert_eq!(edit.get("documentChanges"), None, "{edit}");
            assert_eq!(edit["changes"].as_object().unwrap().len(), 3);
        } else {
            assert_eq!(edit["documentChanges"].as_array().unwrap().len(), 3);
        }
        client.apply_edit(&edit);
        garden.assert_same_as(&fixture("after"));
        assert!(client.shutdown().success());
    }
}

#[test]
fn an_unsaved_buffer_is_renamed_as_the_editor_shows_it() {
    let garden = before();
    let mut client = Client::start(&garden, Options::default());
    let uri = garden.uri("pages/Untouched.md");
    let text = garden.read("pages/Untouched.md");
    client.open(&uri, &text);
    let text = client.replace(&uri, 5, &text, "typeface:: serif", "é:: 1\ntype:: unsaved");
    let mut params = client.at(&uri, &text, "type:: unsaved", 2);
    params["newName"] = json!("kind");
    let refused = message(client.request_raw("textDocument/rename", params.clone()));
    assert!(
        refused.contains("line 1 of pages/Untouched.md"),
        "{refused}"
    );
    params["newName"] = json!("sort");
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
    assert_eq!(&text[start..end], "type");
    assert_eq!(text[..start].lines().count(), 2);
    assert_eq!(change["edits"][0]["newText"], json!("sort"));
    assert!(client.shutdown().success());
}

#[test]
fn help_describes_the_rename_of_a_property_key() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lsg"))
        .args(["lsp", "help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for phrase in [
        "Rename on a property key",
        "on every key:: line",
        "built-in meaning in Logseq",
        "a block that writes both keys",
    ] {
        assert!(help.contains(phrase), "{phrase} is missing from {help}");
    }
    assert!(!help.contains("later release"), "{help}");
}
