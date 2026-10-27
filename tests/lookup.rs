use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Garden(PathBuf);
impl Garden {
    fn new(files: &[(&str, &str)]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-lookup-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        for (name, text) in files {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        Self(root)
    }
    fn lookup(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_lsg"))
            .arg("lookup")
            .arg("--graph")
            .arg(&self.0)
            .args(["--filename-format", "triple-lowbar"])
            .args(args)
            .stdin(Stdio::null())
            .output()
            .unwrap()
    }
    fn path(&self, relative: &str) -> String {
        format!("{}\n", self.0.join(relative).display())
    }
}
impl Drop for Garden {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn sample() -> Garden {
    Garden::new(&[
        (
            "pages/Tools___Neovim.md",
            "alias:: nvim, [[vim]]\n\n- Editor\n",
        ),
        ("pages/Secret.md", "public:: false\n\n- hidden\n"),
        ("pages/A.md", "alias:: shared\n\n- a\n"),
        ("pages/B.md", "alias:: shared\n\n- b\n"),
        ("journals/2025_02_01.md", "- day\n"),
        ("journals/notes.md", "- not a date\n"),
    ])
}

#[test]
fn prints_the_source_path_for_a_title_alias_or_journal_date() {
    let garden = sample();
    for (name, file) in [
        ("tools/neovim", "pages/Tools___Neovim.md"),
        ("NVIM", "pages/Tools___Neovim.md"),
        ("vim", "pages/Tools___Neovim.md"),
        ("Secret", "pages/Secret.md"),
        ("Feb 1st, 2025", "journals/2025_02_01.md"),
        ("2025-02-01", "journals/2025_02_01.md"),
    ] {
        let result = garden.lookup(&[name]);
        assert_eq!(result.status.code(), Some(0), "{name}: {result:?}");
        assert_eq!(String::from_utf8(result.stdout).unwrap(), garden.path(file));
        assert_eq!(
            String::from_utf8(result.stderr).unwrap(),
            "journals/notes.md: journal left out: journals/notes.md: journal file name is not a date in the journal file-name format\n"
        );
    }
}

#[test]
fn json_reports_every_page_sharing_a_name() {
    let garden = sample();
    let result = garden.lookup(&["shared", "--format", "json"]);
    assert!(result.status.success());
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(stderr.ends_with("2 pages match \"shared\"\n"), "{stderr}");
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["format_version"], 1);
    assert_eq!(value["command_path"], serde_json::json!(["lookup"]));
    assert_eq!(value["query"], "shared");
    assert_eq!(value["diagnostic_count"], 1);
    let pages = value["pages"].as_array().unwrap();
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0]["title"], "A");
    assert_eq!(pages[0]["relative_path"], "pages/A.md");
    assert_eq!(pages[0]["path"], garden.path("pages/A.md").trim_end());
    assert_eq!(pages[0]["kind"], "page");
    assert_eq!(pages[0]["matched_by"], "alias");
    assert_eq!(pages[1]["relative_path"], "pages/B.md");

    let journal: Value =
        serde_json::from_slice(&garden.lookup(&["2025_02_01", "--format", "json"]).stdout).unwrap();
    assert_eq!(journal["pages"][0]["kind"], "journal");
    assert_eq!(journal["pages"][0]["matched_by"], "journal-date");
}

#[test]
fn no_match_returns_one_with_empty_stdout() {
    let garden = sample();
    let result = garden.lookup(&["missing", "--format", "json"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        stderr.starts_with("journals/notes.md: journal left out"),
        "{stderr}"
    );
    assert!(stderr.contains("error: no page title, alias or journal date matches \"missing\""));
}

#[test]
fn missing_garden_returns_one_and_missing_inputs_return_two() {
    let missing = Garden(Path::new("/nonexistent/lsg-lookup-garden").to_path_buf());
    let result = missing.lookup(&["x"]);
    std::mem::forget(missing);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("cannot read the garden")
    );

    for args in [
        &["lookup", "x"][..],
        &["lookup", "--graph", "/tmp"],
        &["lookup", "x", "--graph", "/tmp"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_lsg"))
            .args(args)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2), "{args:?}");
        assert!(result.stdout.is_empty());
        assert!(
            String::from_utf8(result.stderr)
                .unwrap()
                .contains("missing required")
        );
    }
}

#[test]
fn a_page_named_help_is_found_after_a_double_dash() {
    let garden = Garden::new(&[("pages/help.md", "- x\n")]);
    let result = garden.lookup(&["--", "help"]);
    assert!(result.status.success(), "{result:?}");
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        garden.path("pages/help.md")
    );
}

#[test]
fn lookup_leaves_source_files_unchanged() {
    let garden = sample();
    let before = fs::read(garden.0.join("pages/Tools___Neovim.md")).unwrap();
    assert!(garden.lookup(&["nvim"]).status.success());
    assert_eq!(
        fs::read(garden.0.join("pages/Tools___Neovim.md")).unwrap(),
        before
    );
    assert_eq!(fs::read_dir(&garden.0).unwrap().count(), 2);
}
