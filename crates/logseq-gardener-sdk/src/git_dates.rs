//! When each page and journal file was first and last committed, read from the garden's
//! git history in one pass. A page's history continues across renames of its file and of
//! the folders above it.
//!
//! ```no_run
//! use logseq_gardener_sdk::git_dates::GitDates;
//!
//! let (dates, diagnostic) = GitDates::read("path/to/garden");
//! if let Some(dates) = dates {
//!     println!("{:?}", dates.get("pages/mise.md"));
//! }
//! if let Some(message) = diagnostic {
//!     eprintln!("{message}");
//! }
//! ```
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::process::Command;

/// Commit times, in seconds since the Unix epoch, of the first and last commits that
/// changed a file. A commit that only renames the file, without changing its text, leaves
/// both times as they were.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileDates {
    pub created: i64,
    pub updated: i64,
}

/// First and last commit times of every file under `pages/` and `journals/`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitDates {
    files: BTreeMap<String, FileDates>,
}

impl GitDates {
    /// Read the history of the garden at `root`. Returns `None` with a diagnostic when
    /// the garden is not in a git repository or `git` cannot run, and dates with a
    /// diagnostic when the repository is a shallow clone, whose history is short.
    pub fn read(root: impl AsRef<Path>) -> (Option<Self>, Option<String>) {
        let root = root.as_ref();
        let git = |args: &[&str]| Command::new("git").arg("-C").arg(root).args(args).output();
        let not_a_repository =
            || Some("Page dates omitted: the garden is not in a git repository".to_owned());
        let probe = match git(&[
            "rev-parse",
            "--is-inside-work-tree",
            "--is-shallow-repository",
            "--show-prefix",
        ]) {
            Ok(output) if output.status.success() => output,
            Ok(_) => return (None, not_a_repository()),
            Err(error) => {
                return (
                    None,
                    Some(format!("Page dates omitted: git could not run ({error})")),
                );
            }
        };
        let probe = String::from_utf8_lossy(&probe.stdout);
        let mut probe = probe.lines();
        if probe.next() != Some("true") {
            return (None, not_a_repository());
        }
        let shallow = probe.next() == Some("true");
        // The garden folder's path inside the repository, such as `docs/`, or empty.
        let prefix = probe.next().unwrap_or_default().to_owned();
        // The whole repository's history, so that a rename from outside the garden folder
        // is seen as a rename. Parents come after their children, which the rename
        // tracking in `parse` relies on.
        let log = git(&[
            "log",
            "--topo-order",
            "--format=%ct",
            "--name-status",
            "-z",
            "--find-renames",
            "--no-relative",
        ]);
        let output = match log {
            Ok(output) if output.status.success() => output,
            Ok(output) => {
                return (
                    None,
                    Some(format!(
                        "Page dates omitted: git log failed ({})",
                        String::from_utf8_lossy(&output.stderr).trim()
                    )),
                );
            }
            Err(error) => {
                return (
                    None,
                    Some(format!("Page dates omitted: git could not run ({error})")),
                );
            }
        };
        let dates = Self::parse(&String::from_utf8_lossy(&output.stdout), &prefix);
        let diagnostic = shallow.then(|| {
            "Page dates come from a shallow clone and reflect only the history it holds".to_owned()
        });
        (Some(dates), diagnostic)
    }

    /// Parse `git log --topo-order --format=%ct --name-status -z --find-renames` output and
    /// keep the files under `prefix` + `pages/` and `prefix` + `journals/`, by their path
    /// below `prefix`.
    ///
    /// Commits come newest first, so a rename is read before the older commits of the file
    /// it renamed. Each file's dates are filed under its current path: in the commits older
    /// than a rename, the old path stands for the renamed file, and the history of the path
    /// the file was renamed to ends there. The renames of one commit apply together, once
    /// its other changes have been read.
    fn parse(log: &str, prefix: &str) -> Self {
        let mut files: HashMap<String, FileDates> = HashMap::new();
        // An older path and the current path whose history it holds; `None` when the
        // history at that path belongs to a file that no longer exists.
        let mut renamed: HashMap<String, Option<String>> = HashMap::new();
        let current =
            |renamed: &HashMap<String, Option<String>>, path: &str| match renamed.get(path) {
                Some(current) => current.clone(),
                None => Some(path.to_owned()),
            };
        let mut record = |path: Option<String>, time: i64| {
            let Some(path) = path else { return };
            files
                .entry(path)
                .and_modify(|dates| {
                    dates.created = dates.created.min(time);
                    dates.updated = dates.updated.max(time);
                })
                .or_insert(FileDates {
                    created: time,
                    updated: time,
                });
        };
        // The renames of the commit being read: the paths renamed to, and each old path
        // with the current path it now stands for.
        let mut ended: Vec<String> = Vec::new();
        let mut moved: Vec<(String, Option<String>)> = Vec::new();
        let apply = |renamed: &mut HashMap<String, Option<String>>,
                     ended: &mut Vec<String>,
                     moved: &mut Vec<(String, Option<String>)>| {
            for path in ended.drain(..) {
                renamed.insert(path, None);
            }
            renamed.extend(moved.drain(..));
        };
        let mut tokens = log.split('\0');
        let mut time = None;
        while let Some(token) = tokens.next() {
            let token = token.trim_start_matches('\n');
            if token.is_empty() {
                continue;
            }
            if token.bytes().all(|byte| byte.is_ascii_digit()) {
                apply(&mut renamed, &mut ended, &mut moved);
                time = token.parse::<i64>().ok();
                continue;
            }
            // A status letter, then one path, or two for a rename or copy.
            let Some(path) = tokens.next() else { break };
            let two_paths = token.starts_with(['R', 'C']);
            let target = if two_paths { tokens.next() } else { None };
            let Some(time) = time else { continue };
            match target {
                Some(target) => {
                    let file = current(&renamed, target);
                    let unchanged = token == "R100";
                    if token.starts_with('R') {
                        ended.push(target.to_owned());
                        moved.push((path.to_owned(), file.clone()));
                    }
                    if !unchanged {
                        record(file, time);
                    }
                }
                None => record(current(&renamed, path), time),
            }
        }
        let files = files
            .into_iter()
            .filter_map(|(path, dates)| {
                let relative = path.strip_prefix(prefix)?;
                (relative.starts_with("pages/") || relative.starts_with("journals/"))
                    .then(|| (relative.to_owned(), dates))
            })
            .collect();
        Self { files }
    }

    /// Dates of a file by its path relative to the garden root, with `/` separators.
    pub fn get(&self, relative_path: &str) -> Option<FileDates> {
        self.files.get(relative_path).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// A fresh directory removed when dropped.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "lsg-git-dates-{name}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn git(root: &Path, args: &[&str], time: &str) {
        let status = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .env("GIT_AUTHOR_DATE", time)
            .env("GIT_COMMITTER_DATE", time)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.com")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.com")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("HOME", root)
            .status()
            .unwrap();
        assert!(status.success());
    }

    #[test]
    fn first_and_last_commit_times_per_file() {
        let temp = TempDir::new("history");
        let root = temp.path();
        git(root, &["init", "-q"], "@1000 +0000");
        fs::create_dir(root.join("pages")).unwrap();
        fs::write(root.join("pages/mise.md"), "- one\n").unwrap();
        fs::write(root.join("pages/tmux.md"), "- one\n").unwrap();
        git(root, &["add", "."], "@1000 +0000");
        git(root, &["commit", "-q", "-m", "add"], "@1000 +0000");
        fs::write(root.join("pages/mise.md"), "- two\n").unwrap();
        fs::write(root.join("README.md"), "x\n").unwrap();
        git(root, &["add", "."], "@2000 +0000");
        git(root, &["commit", "-q", "-m", "edit"], "@2000 +0000");
        let (dates, diagnostic) = GitDates::read(root);
        let dates = dates.unwrap();
        assert_eq!(diagnostic, None);
        assert_eq!(
            dates.get("pages/mise.md"),
            Some(FileDates {
                created: 1000,
                updated: 2000
            })
        );
        assert_eq!(
            dates.get("pages/tmux.md"),
            Some(FileDates {
                created: 1000,
                updated: 1000
            })
        );
        assert_eq!(dates.get("README.md"), None);
    }

    fn commit(root: &Path, time: &str) {
        git(root, &["add", "-A"], time);
        git(root, &["commit", "-q", "-m", "c"], time);
    }

    #[test]
    fn a_renamed_page_file_keeps_its_first_commit_time() {
        let temp = TempDir::new("rename-page");
        let root = temp.path();
        git(root, &["init", "-q"], "@1000 +0000");
        fs::create_dir(root.join("pages")).unwrap();
        fs::write(root.join("pages/mise.md"), "- one\n- two\n- three\n").unwrap();
        commit(root, "@1000 +0000");
        fs::write(root.join("pages/mise.md"), "- one\n- two\n- four\n").unwrap();
        commit(root, "@2000 +0000");
        // A rename alone leaves the dates as they were.
        git(
            root,
            &["mv", "pages/mise.md", "pages/mise___Tasks.md"],
            "@3000 +0000",
        );
        commit(root, "@3000 +0000");
        // A rename that also edits the page is a change to it.
        git(
            root,
            &["mv", "pages/mise___Tasks.md", "pages/Tasks.md"],
            "@4000 +0000",
        );
        fs::write(
            root.join("pages/Tasks.md"),
            "- one\n- two\n- four\n- five\n",
        )
        .unwrap();
        commit(root, "@4000 +0000");
        let (dates, _) = GitDates::read(root);
        let dates = dates.unwrap();
        assert_eq!(
            dates.get("pages/Tasks.md"),
            Some(FileDates {
                created: 1000,
                updated: 4000
            })
        );
    }

    #[test]
    fn a_renamed_graph_folder_keeps_page_dates() {
        let temp = TempDir::new("rename-folder");
        let repo = temp.path();
        git(repo, &["init", "-q"], "@1000 +0000");
        fs::create_dir_all(repo.join("docs/pages")).unwrap();
        fs::create_dir_all(repo.join("docs/journals")).unwrap();
        fs::write(repo.join("docs/pages/lsg.md"), "- one\n").unwrap();
        fs::write(repo.join("docs/journals/2026_11_01.md"), "- day\n").unwrap();
        commit(repo, "@1000 +0000");
        fs::write(repo.join("docs/pages/lsg.md"), "- two\n").unwrap();
        commit(repo, "@2000 +0000");
        git(repo, &["mv", "docs", "garden-docs"], "@3000 +0000");
        commit(repo, "@3000 +0000");
        let (dates, diagnostic) = GitDates::read(repo.join("garden-docs"));
        let dates = dates.unwrap();
        assert_eq!(diagnostic, None);
        assert_eq!(
            dates.get("pages/lsg.md"),
            Some(FileDates {
                created: 1000,
                updated: 2000
            })
        );
        assert_eq!(
            dates.get("journals/2026_11_01.md"),
            Some(FileDates {
                created: 1000,
                updated: 1000
            })
        );
    }

    #[test]
    fn a_page_created_where_another_was_renamed_away_starts_its_own_history() {
        let temp = TempDir::new("rename-reuse");
        let root = temp.path();
        git(root, &["init", "-q"], "@1000 +0000");
        fs::create_dir(root.join("pages")).unwrap();
        fs::write(root.join("pages/Podcast.md"), "- the episode page\n").unwrap();
        commit(root, "@1000 +0000");
        git(
            root,
            &["mv", "pages/Podcast.md", "pages/Podcast___Episode.md"],
            "@2000 +0000",
        );
        commit(root, "@2000 +0000");
        fs::write(root.join("pages/Podcast.md"), "- a new show page\n").unwrap();
        commit(root, "@3000 +0000");
        let (dates, _) = GitDates::read(root);
        let dates = dates.unwrap();
        assert_eq!(
            dates.get("pages/Podcast___Episode.md"),
            Some(FileDates {
                created: 1000,
                updated: 1000
            })
        );
        assert_eq!(
            dates.get("pages/Podcast.md"),
            Some(FileDates {
                created: 3000,
                updated: 3000
            })
        );
    }

    #[test]
    fn a_file_name_with_quotes_has_dates() {
        let temp = TempDir::new("quotes");
        let root = temp.path();
        git(root, &["init", "-q"], "@1000 +0000");
        fs::create_dir(root.join("pages")).unwrap();
        fs::write(root.join("pages/What is \"zoom\"%3F.md"), "- one\n").unwrap();
        commit(root, "@1000 +0000");
        let (dates, _) = GitDates::read(root);
        assert_eq!(
            dates.unwrap().get("pages/What is \"zoom\"%3F.md"),
            Some(FileDates {
                created: 1000,
                updated: 1000
            })
        );
    }

    #[test]
    fn a_directory_outside_git_has_no_dates_and_one_diagnostic() {
        let temp = TempDir::new("outside");
        // Keep git from finding a repository above the temporary directory.
        let root = temp.path().join("garden");
        fs::create_dir(&root).unwrap();
        let (dates, diagnostic) = GitDates::read(&root);
        if dates.is_some() {
            // The system temporary directory is itself inside a repository; nothing to check.
            return;
        }
        assert!(diagnostic.unwrap().contains("not in a git repository"));
    }

    #[test]
    fn parse_keeps_the_oldest_time_as_created() {
        let dates = GitDates::parse(
            &[
                "300",
                "\nM",
                "pages/a.md",
                "200",
                "\nM",
                "pages/a.md",
                "A",
                "pages/b.md",
                "100",
                "\nA",
                "pages/a.md",
                "",
            ]
            .join("\0"),
            "",
        );
        assert_eq!(
            dates.get("pages/a.md"),
            Some(FileDates {
                created: 100,
                updated: 300
            })
        );
        assert_eq!(
            dates.get("pages/b.md"),
            Some(FileDates {
                created: 200,
                updated: 200
            })
        );
    }

    #[test]
    fn parse_applies_the_renames_of_one_commit_together() {
        // In the commit at 200, `b` became `c` and `a` became `b`.
        let dates = GitDates::parse(
            &[
                "200",
                "\nR100",
                "docs/pages/b.md",
                "docs/pages/c.md",
                "R100",
                "docs/pages/a.md",
                "docs/pages/b.md",
                "100",
                "\nA",
                "docs/pages/a.md",
                "A",
                "docs/pages/b.md",
                "A",
                "README.md",
                "",
            ]
            .join("\0"),
            "docs/",
        );
        assert_eq!(
            dates.get("pages/b.md"),
            Some(FileDates {
                created: 100,
                updated: 100
            })
        );
        assert_eq!(
            dates.get("pages/c.md"),
            Some(FileDates {
                created: 100,
                updated: 100
            })
        );
        assert_eq!(dates.get("pages/a.md"), None);
        assert_eq!(dates.get("README.md"), None);
        assert_eq!(dates.get("../README.md"), None);
    }

    #[test]
    fn a_shallow_clone_reports_a_diagnostic() {
        let temp = TempDir::new("shallow");
        let source = temp.path().join("source");
        fs::create_dir_all(source.join("pages")).unwrap();
        git(&source, &["init", "-q"], "@1000 +0000");
        for (time, text) in [("@1000 +0000", "- one\n"), ("@2000 +0000", "- two\n")] {
            fs::write(source.join("pages/a.md"), text).unwrap();
            git(&source, &["add", "."], time);
            git(&source, &["commit", "-q", "-m", "c"], time);
        }
        let clone = temp.path().join("clone");
        let url = format!("file://{}", source.display());
        git(
            temp.path(),
            &["clone", "-q", "--depth", "1", &url, clone.to_str().unwrap()],
            "@3000 +0000",
        );
        let (dates, diagnostic) = GitDates::read(&clone);
        assert_eq!(
            dates.unwrap().get("pages/a.md"),
            Some(FileDates {
                created: 2000,
                updated: 2000
            })
        );
        assert!(diagnostic.unwrap().contains("shallow clone"));
    }
}
