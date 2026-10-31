use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-publish-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn write(&self, name: &str, text: &str) {
        let path = self.0.join("garden").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn publish(&self, output: &Path, extra: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_lsg"))
            .args([
                OsStr::new("publish"),
                OsStr::new("--graph"),
                self.0.join("garden").as_os_str(),
                OsStr::new("--output"),
                output.as_os_str(),
            ])
            .args(["--filename-format", "triple-lowbar"])
            .args(extra)
            .stdin(Stdio::null())
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, path: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, result);
            } else {
                result.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}
#[test]
fn cli_publishes_links_assets_and_diagnostics_without_excluded_content_or_source_changes() {
    let f = Fixture::new();
    f.write("pages/Notes___Start.md", "- [[Notes/Other]] [[Notes/Private/Hidden]]\n- {{embed [[Notes/Private/Hidden]]}}\n- ((12345678-1234-1234-1234-123456789abc))\n- ![visible](../assets/shared.png)\n");
    f.write("pages/Notes___Other.md", "- SELECTED_PAGE\n");
    f.write("pages/Extras___Included.md", "- SECOND_INCLUDE\n");
    f.write("pages/Notes___Private___Hidden.md", "- EXCLUDED_PAGE_SENTINEL\n  id:: 12345678-1234-1234-1234-123456789abc\n  - EXCLUDED_BLOCK_SENTINEL\n- ![excluded](../assets/excluded.png)\n");
    f.write("pages/Notes___Draft.md", "- EXCLUDED_DRAFT_SENTINEL\n");
    f.write(
        "pages/Notes___Withheld.md",
        "- WITHHELD_SENTINEL\n  public:: false\n",
    );
    f.write("journals/2026_10_07.md", "- JOURNAL_SENTINEL\n");
    f.write("assets/shared.png", "SELECTED_ASSET_CONTENT");
    f.write("assets/excluded.png", "EXCLUDED_ASSET_SENTINEL");
    let before = files(&f.0.join("garden"));
    let output = f.0.join("site");
    let result = f.publish(
        &output,
        &[
            "--include",
            "Notes",
            "--include",
            "Extras",
            "--include",
            "Notes/Private",
            "--exclude",
            "Notes/Private",
            "--exclude",
            "Notes/Draft",
            "--format",
            "json",
        ],
    );
    assert!(result.status.success(), "{result:?}");
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["format_version"], 1);
    assert_eq!(report["command_path"], serde_json::json!(["publish"]));
    assert_eq!(report["pages"], 3);
    assert_eq!(report["assets"], 1);
    assert_eq!(report["withheld_pages"], 1);
    assert_eq!(report["skipped_journals"], 1);
    assert!(report["diagnostic_count"].as_u64().unwrap() > 0);
    assert_eq!(
        report["index_path"],
        output.join("index.html").to_string_lossy().as_ref()
    );
    assert!(!result.stderr.is_empty());
    let generated = files(&output);
    let index = String::from_utf8_lossy(generated.get(Path::new("index.html")).unwrap());
    assert!(index.contains("Notes/Other") && index.contains("Extras/Included"));
    for text in generated.values().map(|v| String::from_utf8_lossy(v)) {
        for excluded in [
            "EXCLUDED_PAGE_SENTINEL",
            "EXCLUDED_BLOCK_SENTINEL",
            "EXCLUDED_ASSET_SENTINEL",
            "EXCLUDED_DRAFT_SENTINEL",
            "WITHHELD_SENTINEL",
            "JOURNAL_SENTINEL",
        ] {
            assert!(!text.contains(excluded), "{excluded}");
        }
    }
    let start = generated
        .values()
        .find(|v| String::from_utf8_lossy(v).contains("<h1>Notes/Start</h1>"))
        .unwrap();
    let start = String::from_utf8_lossy(start);
    assert!(start.contains("href=\"p-") && start.contains("<img"));
    assert!(generated.values().any(|v| v == b"SELECTED_ASSET_CONTENT"));
    assert_eq!(before, files(&f.0.join("garden")));
    let human = f.publish(&f.0.join("human-site"), &["--include", "Extras"]);
    assert!(human.status.success());
    // The fixture garden has no git history, so the only diagnostic is the missing dates.
    let stderr = String::from_utf8_lossy(&human.stderr);
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains("not in a git repository"), "{stderr}");
    assert!(String::from_utf8_lossy(&human.stdout).contains("Published 1 pages"));
}
#[test]
fn errors_have_empty_stdout_and_preserve_fatal_vs_argument_exits() {
    let f = Fixture::new();
    f.write("pages/Notes.md", "- visible\n");
    for flag in ["--graph", "--output", "--filename-format"] {
        let result = Command::new(env!("CARGO_BIN_EXE_lsg"))
            .args(["publish", flag])
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("help"));
    }
    for extra in [
        vec!["--include", "Notes//Bad"],
        vec!["--exclude", " Notes"],
        vec!["--unknown"],
    ] {
        let result = f.publish(&f.0.join("site"), &extra);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert!(!f.0.join("site").exists());
    }
    fs::create_dir(f.0.join("existing")).unwrap();
    for (output, expected) in [
        (f.0.join("existing"), "already exists"),
        (f.0.join("garden/site"), "outside the source garden"),
        (f.0.join("missing/site"), "publication failed"),
    ] {
        let result = f.publish(&output, &["--include", "Notes", "--format", "json"]);
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains(expected));
    }
    f.write("pages/duplicate.md", "title:: Notes\n- duplicate\n");
    let result = f.publish(&f.0.join("collision"), &["--include", "Notes"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("ambiguous"));
    assert!(!f.0.join("collision").exists());
    fs::remove_dir_all(f.0.join("garden")).unwrap();
    let result = f.publish(&f.0.join("missing-source"), &[]);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("publication failed"));
}
#[cfg(unix)]
#[test]
fn native_paths_and_controls_are_preserved_for_files_and_escaped_in_reports() {
    #[cfg(target_os = "linux")]
    use std::os::unix::ffi::OsStringExt;
    let f = Fixture::new();
    f.write("pages/Notes.md", "- visible\n");
    #[cfg(target_os = "linux")]
    let output =
        f.0.join(std::ffi::OsString::from_vec(b"site-\xff-\x1b[31m".to_vec()));
    #[cfg(not(target_os = "linux"))]
    let output = f.0.join("site-庭-\x1b[31m");
    let result = f.publish(&output, &["--include", "Notes"]);
    assert!(result.status.success(), "{result:?}");
    assert!(!result.stdout.contains(&27));
    assert!(output.join("index.html").exists());
    #[cfg(target_os = "linux")]
    let json_output =
        f.0.join(std::ffi::OsString::from_vec(b"json-\xff-\x1b[31m".to_vec()));
    #[cfg(not(target_os = "linux"))]
    let json_output = f.0.join("json-庭-\x1b[31m");
    let result = f.publish(&json_output, &["--include", "Notes", "--format", "json"]);
    assert!(result.status.success());
    assert!(!result.stdout.contains(&27));
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(
        report["output_directory"],
        json_output.to_string_lossy().as_ref()
    );
    let result = f.publish(&output, &["--include", "Notes"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(!result.stderr.contains(&27));
}

#[test]
fn explicit_inputs_legacy_format_and_empty_selection() {
    let f = Fixture::new();
    f.write("pages/Notes%2FStart.md", "- visible\n");
    let graph = f.0.join("garden");
    for args in [
        vec![OsStr::new("publish")],
        vec![
            OsStr::new("publish"),
            OsStr::new("--graph"),
            graph.as_os_str(),
        ],
        vec![
            OsStr::new("publish"),
            OsStr::new("--graph"),
            graph.as_os_str(),
            OsStr::new("--output"),
            f.0.as_os_str(),
        ],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_lsg"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("missing required"));
    }
    let result = f.publish(&f.0.join("empty"), &["--format", "json"]);
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["pages"], 0);
    // The index page, the page list, the shell, outline, references and All pages scripts,
    // the right sidebar's script and styles, the View as slides script, and the graph view's
    // page, script, worker, styles and data file, and the flashcard review's script,
    // styles, deck list and `All` deck, and the site manifest lsg-site.json.
    assert_eq!(files(&f.0.join("empty")).len(), 19);
    let output = f.0.join("legacy");
    let result = Command::new(env!("CARGO_BIN_EXE_lsg"))
        .arg("publish")
        .arg("--graph")
        .arg(graph)
        .arg("--output")
        .arg(&output)
        .args([
            "--filename-format",
            "legacy",
            "--include",
            "Notes",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["pages"], 1);
    assert!(
        fs::read_to_string(output.join("index.html"))
            .unwrap()
            .contains("Notes/Start")
    );
}

#[cfg(unix)]
#[test]
fn closed_diagnostics_pipe_keeps_success_json() {
    use std::os::fd::OwnedFd;
    use std::os::unix::net::UnixStream;
    let f = Fixture::new();
    f.write("pages/Notes.md", "- [[Missing]]\n");
    let (writer, reader) = UnixStream::pair().unwrap();
    drop(reader);
    let output = f.0.join("site");
    let result = Command::new(env!("CARGO_BIN_EXE_lsg"))
        .arg("publish")
        .arg("--graph")
        .arg(f.0.join("garden"))
        .arg("--output")
        .arg(&output)
        .args([
            "--filename-format",
            "triple-lowbar",
            "--include",
            "Notes",
            "--format",
            "json",
        ])
        .stderr(Stdio::from(OwnedFd::from(writer)))
        .output()
        .unwrap();
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(report["diagnostic_count"].as_u64().unwrap() > 0);
}

#[cfg(unix)]
#[test]
fn unicode_control_paths_produce_valid_json_without_terminal_controls() {
    let f = Fixture::new();
    f.write("pages/Notes.md", "- visible\n");
    for control in ['\u{7f}', '\u{85}'] {
        let output = f.0.join(format!("site-{control}"));
        let result = f.publish(&output, &["--include", "Notes", "--format", "json"]);
        assert!(result.status.success(), "{result:?}");
        let text = std::str::from_utf8(&result.stdout).unwrap();
        assert!(!text.contains(control));
        let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(
            report["output_directory"],
            output.to_string_lossy().as_ref()
        );
        assert!(output.join("index.html").exists());
    }
}

#[test]
fn selected_media_renders_without_leaking_private_or_excluded_recordings() {
    let f = Fixture::new();
    f.write("pages/Podcast___Episode.md", "- ![Local recording](../assets/song.mp3)\n- ![Remote recording](https://media.example/selected.mp3)\n- ![Artwork](https://media.example/selected.gif)\n- [Plain download](https://media.example/download.mp3)\n- ![Wave recording](../assets/song.wav) ![Ogg recording](../assets/song.ogg)\n");
    f.write("pages/Podcast___Private___Hidden.md", "- EXCLUDED_MEDIA_TEXT\n- ![excluded](../assets/excluded.mp3)\n- ![remote](https://media.example/EXCLUDED_MEDIA_URL.mp3)\n");
    f.write("pages/Podcast___Withheld.md", "public:: false\n- WITHHELD_MEDIA_TEXT\n- ![private](../assets/private.ogg)\n- ![remote](https://media.example/PRIVATE_MEDIA_URL.gif)\n");
    f.write("assets/song.mp3", "SELECTED_MP3");
    f.write("assets/song.wav", "SELECTED_WAV");
    f.write("assets/song.ogg", "SELECTED_OGG");
    f.write("assets/excluded.mp3", "EXCLUDED_MEDIA_FILE");
    f.write("assets/private.ogg", "PRIVATE_MEDIA_FILE");
    let before = files(&f.0.join("garden"));
    let out = f.0.join("site");
    let response = f.publish(
        &out,
        &[
            "--include",
            "Podcast",
            "--exclude",
            "Podcast/Private",
            "--format",
            "json",
        ],
    );
    assert!(response.status.success(), "{:?}", response);
    let report: serde_json::Value = serde_json::from_slice(&response.stdout).unwrap();
    assert_eq!(report["pages"], 1);
    assert_eq!(report["assets"], 3);
    assert_eq!(report["withheld_pages"], 1);
    let generated = files(&out);
    let page = generated
        .iter()
        .find(|(p, _)| {
            p.extension().is_some_and(|e| e == "html")
                && !["index.html", "graph.html"].contains(&p.file_name().unwrap().to_str().unwrap())
        })
        .unwrap()
        .1;
    let html = String::from_utf8_lossy(page);
    assert_eq!(html.matches("<audio controls").count(), 4);
    assert!(html.contains("aria-label=\"Local recording\""));
    assert!(html.contains("<img src=\"https://media.example/selected.gif\" alt=\"Artwork\" title=\"Artwork\" loading=\"lazy\">"));
    assert!(html.contains("<a class=\"external-link\" target=\"_blank\" rel=\"noopener noreferrer\" href=\"https://media.example/download.mp3\">Plain download</a>"));
    assert!(html.contains(" download>Local recording</a>"));
    assert!(html.contains(
        "<a href=\"https://media.example/selected.mp3\">Open recording: Remote recording</a>"
    ));
    for content in generated.values() {
        let content = String::from_utf8_lossy(content);
        for sentinel in [
            "EXCLUDED_MEDIA_TEXT",
            "EXCLUDED_MEDIA_URL",
            "EXCLUDED_MEDIA_FILE",
            "WITHHELD_MEDIA_TEXT",
            "PRIVATE_MEDIA_URL",
            "PRIVATE_MEDIA_FILE",
        ] {
            assert!(!content.contains(sentinel), "{sentinel}");
        }
    }
    assert_eq!(files(&f.0.join("garden")), before);
}

#[test]
fn cli_stylesheet_is_copied_and_invalid_input_fails_without_output() {
    let f = Fixture::new();
    f.write("pages/Notes.md", "- Visible\n");
    let css = fs::canonicalize(&f.0).unwrap().join("theme.css");
    fs::write(&css, "body { color: purple; }\n").unwrap();
    let output = f.0.join("styled");
    let result = f.publish(
        &output,
        &[
            "--include",
            "Notes",
            "--stylesheet",
            css.to_str().unwrap(),
            "--format",
            "json",
        ],
    );
    assert!(result.status.success(), "{result:?}");
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["pages"], 1);
    assert_eq!(report["assets"], 0);
    assert_eq!(
        fs::read(output.join("custom.css")).unwrap(),
        fs::read(&css).unwrap()
    );
    for (path, contents) in files(&output) {
        if path.extension().is_some_and(|ext| ext == "html") {
            assert!(
                String::from_utf8(contents)
                    .unwrap()
                    .contains("href=\"custom.css\"")
            );
        }
    }
    let failed_output = f.0.join("failed");
    let result = f.publish(
        &failed_output,
        &["--include", "Notes", "--stylesheet", "missing.css"],
    );
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("missing.css"));
    assert!(!failed_output.exists());
    let help = Command::new(env!("CARGO_BIN_EXE_lsg"))
        .args(["publish", "help"])
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--stylesheet PATH"));
}

#[test]
fn cli_homepage_selects_alias_and_rejects_withheld_targets() {
    let f = Fixture::new();
    f.write("pages/Home.md", "alias:: Welcome\n- HOMEPAGE_CONTENT\n");
    f.write("pages/Hidden.md", "public:: false\n- HIDDEN_CONTENT\n");
    let output = f.0.join("site");
    let result = f.publish(&output, &["--include", "Home", "--homepage", "Welcome"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        fs::read_to_string(output.join("index.html"))
            .unwrap()
            .contains("HOMEPAGE_CONTENT")
    );
    assert!(output.join("pages.html").exists());
    let rejected = f.0.join("rejected");
    let result = f.publish(&rejected, &["--include", "Hidden", "--homepage", "Hidden"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(!rejected.exists());
}

#[test]
fn search_flag_emits_only_opt_in_external_artifacts() {
    let f = Fixture::new();
    f.write("pages/Notes.md", "- Searchable original text\n");
    let output = f.0.join("search");
    let result = f.publish(&output, &["--include", "Notes", "--search"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let emitted = files(&output);
    let manifests: Vec<_> = emitted
        .keys()
        .filter(|path| path.to_string_lossy().starts_with("lsg-search-manifest-"))
        .collect();
    assert_eq!(manifests.len(), 1);
    assert!(
        emitted
            .keys()
            .any(|path| path.to_string_lossy().starts_with("lsg-search-engine-"))
    );
    assert!(
        String::from_utf8_lossy(&emitted[Path::new("index.html")]).contains("rel=\"lsg-search\"")
    );
}

#[test]
fn cli_plugin_adds_files_and_invalid_manifests_fail_without_output() {
    let f = Fixture::new();
    f.write("pages/Notes.md", "- Visible\n");
    let manifest = |api: u8| {
        let folder = f.0.join(format!("player-{api}"));
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("player.mjs"), "export default {};\n").unwrap();
        let path = folder.join("lsg-plugin.json");
        fs::write(
            &path,
            format!(r#"{{"id":"player","apiVersion":{api},"version":"1","module":"player.mjs","activation":"after-load"}}"#),
        )
        .unwrap();
        path
    };
    let report = |output: &Path, extra: &[&str]| {
        let mut args = vec!["--include", "Notes", "--format", "json"];
        args.extend(extra);
        let result = f.publish(output, &args);
        assert!(result.status.success(), "{result:?}");
        let mut report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        report
            .as_object_mut()
            .unwrap()
            .retain(|key, _| !key.contains("path") && !key.contains("directory"));
        report
    };
    let valid = manifest(1);
    let plain = report(&f.0.join("plain"), &[]);
    let output = f.0.join("with-plugin");
    assert_eq!(
        report(&output, &["--plugin", valid.to_str().unwrap()]),
        plain
    );
    assert!(output.join("plugins/player/player.mjs").is_file());
    let site: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("lsg-site.json")).unwrap()).unwrap();
    assert_eq!(site["plugins"][0]["module"], "plugins/player/player.mjs");
    assert!(f.0.join("plain/lsg-site.json").is_file());

    let invalid = manifest(2);
    let failed_output = f.0.join("failed");
    let result = f.publish(
        &failed_output,
        &["--include", "Notes", "--plugin", invalid.to_str().unwrap()],
    );
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains(invalid.to_str().unwrap()), "{stderr}");
    assert!(stderr.contains("unsupported apiVersion 2"), "{stderr}");
    assert!(!failed_output.exists());

    let help = Command::new(env!("CARGO_BIN_EXE_lsg"))
        .args(["publish", "help", "section", "plugins"])
        .output()
        .unwrap();
    assert!(help.status.success());
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(help.contains("--plugin PATH"), "{help}");
    assert!(help.contains("plugins/<id>/"), "{help}");
    assert!(help.contains("docs/publishing.md"), "{help}");
}
