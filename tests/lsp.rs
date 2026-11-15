//! End-to-end tests of `lsg lsp`: the built binary, a copy of the fixture garden, and
//! JSON-RPC over standard input and output.

#[path = "lsp/client.rs"]
mod client;

use client::{Client, Garden, Options};
use serde_json::{Value, json};

const APPLE: &str = "pages/Fruit___Apple.md";
const RECIPES: &str = "pages/Recipes.md";

fn status(client: &mut Client) -> Value {
    client.request("lsg/status", Value::Null)
}

fn occurrence(client: &mut Client, uri: &str, text: &str, needle: &str) -> Value {
    let params = client.at(uri, text, needle, 3);
    client.request("lsg/occurrence", params)
}

#[test]
fn a_session_initializes_reports_progress_and_exits_cleanly() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());

    let capabilities = &client.initialize_result["capabilities"];
    assert_eq!(capabilities["positionEncoding"], "utf-8");
    assert_eq!(capabilities["textDocumentSync"]["openClose"], true);
    // Incremental changes.
    assert_eq!(capabilities["textDocumentSync"]["change"], 2);
    for formatting in [
        "documentFormattingProvider",
        "documentRangeFormattingProvider",
        "documentOnTypeFormattingProvider",
    ] {
        assert!(capabilities.get(formatting).is_none(), "{formatting}");
    }
    assert_eq!(client.initialize_result["serverInfo"]["name"], "lsg");

    // A request sent right after `initialized` gets its answer once the garden is loaded.
    let status = status(&mut client);
    assert_eq!(status["documents"], 7);
    assert_eq!(status["pages"], 5);
    assert_eq!(status["journals"], 2);
    assert_eq!(status["openDocuments"], 0);
    assert_eq!(status["filenameFormat"], "triple-lowbar");
    assert_eq!(status["journalTitleFormat"], "MMM do, yyyy");
    assert_eq!(status["positionEncoding"], "utf-8");
    assert_eq!(status["fileWatching"], "client");
    assert_eq!(status["root"], garden.root.to_string_lossy().as_ref());

    // Progress: a token the server created, a beginning and an end.
    let created = client.wait_for_server_request("window/workDoneProgress/create");
    let mut kinds = Vec::new();
    while kinds.last().map(String::as_str) != Some("end") {
        let progress = client.wait_for_notification("$/progress");
        assert_eq!(progress["token"], created["token"]);
        kinds.push(progress["value"]["kind"].as_str().unwrap().to_owned());
    }
    assert_eq!(kinds.first().map(String::as_str), Some("begin"));

    // The server asked the client to watch the garden's files.
    let registration = client.wait_for_server_request("client/registerCapability");
    let registration = &registration["registrations"][0];
    assert_eq!(registration["method"], "workspace/didChangeWatchedFiles");
    let patterns: Vec<&str> = registration["registerOptions"]["watchers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|watcher| watcher["globPattern"]["pattern"].as_str().unwrap())
        .collect();
    assert_eq!(
        patterns,
        ["pages/**/*.md", "journals/**/*.md", "logseq/config.edn"]
    );

    assert_eq!(
        client
            .request_raw("textDocument/formatting", json!({}))
            .unwrap_err()["code"],
        -32601
    );
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn an_open_buffer_is_ahead_of_the_file_until_it_closes() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());
    let uri = garden.uri(RECIPES);
    let text = garden.read(RECIPES);

    // From disk: an alias resolves to its page.
    let found = occurrence(&mut client, &uri, &text, "[[Malus]]");
    assert_eq!(found["kind"], "pageLink");
    assert_eq!(found["target"], json!({"type": "page", "name": "Malus"}));
    assert_eq!(found["resolved"]["uri"], garden.uri(APPLE));
    assert_eq!(found["resolved"]["title"], "Fruit/Apple");

    // An unsaved edit changes what the link names.
    client.open(&uri, &text);
    let text = client.replace(&uri, 2, &text, "[[Malus]]", "[[Fruit/Quince]]");
    let found = occurrence(&mut client, &uri, &text, "[[Fruit/Quince]] from");
    assert_eq!(
        found["resolved"],
        json!({"type": "pageWithoutFile", "name": "fruit/quince"})
    );
    assert_eq!(status(&mut client)["openDocuments"], 1);

    // A second edit, later in the document, lands at the position the first one left.
    let text = client.replace(&uri, 3, &text, "[[2026-01-03]]", "[[2026-01-04]]");
    let found = occurrence(&mut client, &uri, &text, "[[2026-01-04]]");
    assert_eq!(
        found["resolved"],
        json!({"type": "journalWithoutFile", "date": "2026-01-04"})
    );

    // The file on disk still holds the old text, and closing goes back to it.
    assert!(garden.read(RECIPES).contains("[[Malus]]"));
    client.close(&uri);
    let text = garden.read(RECIPES);
    let found = occurrence(&mut client, &uri, &text, "[[Malus]]");
    assert_eq!(found["resolved"]["title"], "Fruit/Apple");
    assert_eq!(status(&mut client)["openDocuments"], 0);
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn occurrences_report_ranges_and_targets_in_both_encodings() {
    let garden = Garden::copy();
    let text = garden.read(RECIPES);
    let uri = garden.uri(RECIPES);
    // The link sits after `Café` and an emoji on its line.
    let line = text.lines().position(|line| line.contains("Café")).unwrap();
    let column = |utf8: bool, needle: &str| {
        client::position_of(&text, needle, 0, utf8)["character"]
            .as_u64()
            .unwrap()
    };
    for utf8 in [true, false] {
        let options = Options {
            utf8,
            ..Options::default()
        };
        let mut client = Client::start(&garden, options);
        let encoding = if utf8 { "utf-8" } else { "utf-16" };
        assert_eq!(
            client.initialize_result["capabilities"]["positionEncoding"],
            encoding
        );
        let found = occurrence(&mut client, &uri, &text, "[[Fruit/Apple]] and");
        let start = column(utf8, "[[Fruit/Apple]] and");
        assert_eq!(
            found["range"],
            json!({
                "start": {"line": line, "character": start},
                "end": {"line": line, "character": start + 15},
            }),
            "{encoding}"
        );
        assert_eq!(found["nameRange"]["start"]["character"], start + 2);
        assert_eq!(found["nameRange"]["end"]["character"], start + 13);
        // The emoji is four UTF-8 units and two UTF-16 units, and `é` is two and one.
        assert_eq!(
            column(true, "[[Fruit/Apple]] and") - column(false, "[[Fruit/Apple]] and"),
            3
        );

        let found = occurrence(
            &mut client,
            &uri,
            &text,
            "((66f1a2b0-0000-4000-8000-000000000002))",
        );
        assert_eq!(found["kind"], "blockEmbed");
        assert_eq!(found["resolved"]["type"], "block");
        assert_eq!(found["resolved"]["uri"], garden.uri(APPLE));
        // A block starts at the start of its bullet's line, before the indentation.
        let apple = garden.read(APPLE);
        assert_eq!(
            found["resolved"]["range"]["start"],
            client::position_of(&apple, "\t- Store apart", 0, utf8)
        );
        let found = occurrence(
            &mut client,
            &uri,
            &text,
            "((66f1a2b0-0000-4000-8000-00000000dead))",
        );
        assert_eq!(found["resolved"], json!({"type": "blockNotFound"}));
        let found = occurrence(&mut client, &uri, &text, "#[[multi word]]");
        assert_eq!(found["kind"], "tag");
        assert_eq!(found["resolved"]["uri"], garden.uri("pages/multi word.md"));
        let found = occurrence(&mut client, &uri, &text, "serves::");
        assert_eq!(found["kind"], "propertyKey");
        assert_eq!(found["resolved"], json!({"type": "property", "count": 1}));
        let params = client.at(&uri, &text, "Peel four", 2);
        assert_eq!(client.request("lsg/occurrence", params), Value::Null);
        assert_eq!(client.shutdown().code(), Some(0));
    }
}

#[test]
fn file_changes_the_client_reports_reach_the_index() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());
    assert_eq!(status(&mut client)["documents"], 7);

    // Created.
    garden.write(
        "pages/Fruit___Quince.md",
        "alias:: Cydonia\n\n- Hard and sour.\n",
    );
    client.files_changed(&[(&garden.uri("pages/Fruit___Quince.md"), 1)]);
    assert_eq!(status(&mut client)["documents"], 8);
    let uri = garden.uri(RECIPES);
    let text = garden.read(RECIPES);
    let found = occurrence(&mut client, &uri, &text, "[[Fruit/Quince]]");
    assert_eq!(
        found["resolved"]["uri"],
        garden.uri("pages/Fruit___Quince.md")
    );

    // Changed: the page gives up an alias.
    let apple = garden
        .read(APPLE)
        .replace("alias:: Malus, [[Pomme]]", "alias:: [[Pomme]]");
    garden.write(APPLE, &apple);
    client.files_changed(&[(&garden.uri(APPLE), 2)]);
    let found = occurrence(&mut client, &uri, &text, "[[Malus]]");
    assert_eq!(
        found["resolved"],
        json!({"type": "pageWithoutFile", "name": "malus"})
    );

    // Deleted, and a hidden file that changes nothing.
    garden.remove("pages/Fruit___Quince.md");
    garden.write("pages/archive/More.md", "- hidden\n");
    client.files_changed(&[
        (&garden.uri("pages/Fruit___Quince.md"), 3),
        (&garden.uri("pages/archive/More.md"), 1),
    ]);
    assert_eq!(status(&mut client)["documents"], 7);
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn the_server_watches_files_for_a_client_that_cannot() {
    let garden = Garden::copy();
    let options = Options {
        client_watches: false,
        ..Options::default()
    };
    let mut client = Client::start(&garden, options);
    assert_eq!(status(&mut client)["fileWatching"], "server");
    assert_eq!(status(&mut client)["documents"], 7);

    garden.write("journals/2026_01_04.md", "- A new day with [[Fruit]].\n");
    client.wait_until("the new journal is indexed", |client| {
        (status(client)["documents"] == 8).then_some(())
    });
    garden.remove("journals/2026_01_04.md");
    client.wait_until("the deleted journal leaves the index", |client| {
        (status(client)["documents"] == 7).then_some(())
    });
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn a_config_change_reloads_the_settings_and_the_index() {
    for client_watches in [true, false] {
        let garden = Garden::copy();
        let options = Options {
            client_watches,
            ..Options::default()
        };
        let mut client = Client::start(&garden, options);
        let uri = garden.uri(RECIPES);
        let text = garden.read(RECIPES);
        assert_eq!(status(&mut client)["filenameFormat"], "triple-lowbar");
        // An open buffer keeps its text through the reload.
        client.open(&uri, &text);
        let text = client.replace(
            &uri,
            2,
            &text,
            "Apple cake",
            "Apple cake for [[Fruit___Pear]]",
        );

        garden.write(
            "logseq/config.edn",
            "{:journal/page-title-format \"yyyy-MM-dd\"}\n",
        );
        if client_watches {
            client.files_changed(&[(&garden.uri("logseq/config.edn"), 2)]);
        }
        let status = client.wait_until("the settings are read again", |client| {
            let status = status(client);
            (status["filenameFormat"] == "legacy").then_some(status)
        });
        assert_eq!(status["journalTitleFormat"], "yyyy-MM-dd");
        assert_eq!(status["openDocuments"], 1);
        // Under the legacy format the file name `Fruit___Pear` is the title as written.
        let found = occurrence(&mut client, &uri, &text, "[[Fruit___Pear]]");
        assert_eq!(
            found["resolved"]["uri"],
            garden.uri("pages/Fruit___Pear.md")
        );
        let found = occurrence(&mut client, &uri, &text, "[[Fruit/Apple]]");
        assert_eq!(found["resolved"]["type"], "pageWithoutFile");
        assert_eq!(client.shutdown().code(), Some(0));
    }
}

#[test]
fn a_folder_inside_the_garden_finds_the_root() {
    let garden = Garden::copy();
    let mut client = Client::spawn(true);
    client
        .initialize(&garden.path("pages"), Options::default())
        .unwrap();
    client.notify("initialized", json!({}));
    assert_eq!(
        status(&mut client)["root"],
        garden.root.to_string_lossy().as_ref()
    );
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn a_folder_that_is_no_garden_gets_an_error_the_client_can_show() {
    let folder = Garden::empty();
    folder.write("pages/Note.md", "- text\n");
    let mut client = Client::spawn(true);
    let error = client
        .initialize(&folder.root, Options::default())
        .unwrap_err();
    let message = error["message"].as_str().unwrap();
    assert!(message.contains("logseq/config.edn"), "{message}");
    assert!(
        message.contains(folder.root.to_string_lossy().as_ref()),
        "{message}"
    );
    let (status, stderr) = client.disconnect();
    assert_eq!(status.code(), Some(1));
    assert!(stderr.contains("logseq/config.edn"), "{stderr}");
}

#[test]
fn closing_the_connection_without_shutdown_exits_with_one() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());
    assert_eq!(status(&mut client)["documents"], 7);
    let (status, stderr) = client.disconnect();
    assert_eq!(status.code(), Some(1));
    assert!(stderr.contains("without shutdown"), "{stderr}");
}
