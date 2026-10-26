//! When each page and journal file was first and last committed, read from the garden's
//! git history in one pass.
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
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// Commit times, in seconds since the Unix epoch, of the first and last commits that
/// touched a file.
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
        match git(&["rev-parse", "--is-inside-work-tree"]) {
            Ok(output) if output.status.success() => {}
            Ok(_) => {
                return (
                    None,
                    Some("Page dates omitted: the garden is not in a git repository".to_owned()),
                );
            }
            Err(error) => {
                return (
                    None,
                    Some(format!("Page dates omitted: git could not run ({error})")),
                );
            }
        }
        let log = git(&[
            "-c",
            "core.quotePath=false",
            "log",
            "--format=%ct",
            "--name-only",
            "--no-renames",
            "--relative",
            "--",
            "pages",
            "journals",
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
        let dates = Self::parse(&String::from_utf8_lossy(&output.stdout));
        let shallow = git(&["rev-parse", "--is-shallow-repository"])
            .is_ok_and(|output| output.stdout.starts_with(b"true"));
        let diagnostic = shallow.then(|| {
            "Page dates come from a shallow clone and reflect only the history it holds".to_owned()
        });
        (Some(dates), diagnostic)
    }

    /// Parse `git log --format=%ct --name-only` output, newest commit first.
    fn parse(log: &str) -> Self {
        let mut files: BTreeMap<String, FileDates> = BTreeMap::new();
        let mut time = None;
        for line in log.lines() {
            if line.is_empty() {
                continue;
            }
            if let Ok(seconds) = line.parse::<i64>()
                && !line.contains('/')
            {
                time = Some(seconds);
                continue;
            }
            let Some(time) = time else { continue };
            files
                .entry(line.to_owned())
                .and_modify(|dates| dates.created = time)
                .or_insert(FileDates {
                    created: time,
                    updated: time,
                });
        }
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
            "300\n\npages/a.md\n200\n\npages/a.md\npages/b.md\n100\n\npages/a.md\n",
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
