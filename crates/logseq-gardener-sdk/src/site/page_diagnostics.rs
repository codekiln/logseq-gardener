//! Page diagnostics, each reported once under the file that holds the problem.
//!
//! A page shows blocks of other pages in its embeds, Linked References and query
//! results. The renderer names the file such a block comes from at the start of its
//! diagnostic, as `<file>: <message>`. That file renders as a page of its own and
//! reports the same problem there, so a borrowed diagnostic is moved to that file and
//! kept only when the file does not already report it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::SiteDiagnostic;

/// Diagnostics of the rendered pages, in the order the pages report them.
pub(super) struct PageDiagnostics {
    /// Source files of the rendered pages, as the renderer prints them.
    files: BTreeSet<String>,
    /// Each diagnostic, and whether it came from a block of another file.
    entries: Vec<(SiteDiagnostic, bool)>,
}

impl PageDiagnostics {
    pub(super) fn new<'a>(files: impl IntoIterator<Item = &'a Path>) -> Self {
        Self {
            files: files
                .into_iter()
                .map(|file| file.display().to_string())
                .collect(),
            entries: Vec::new(),
        }
    }

    /// Records `message`, reported while rendering the page in `page`.
    pub(super) fn push(&mut self, page: &Path, message: String) {
        let borrowed = message.match_indices(": ").find_map(|(at, _)| {
            self.files
                .contains(&message[..at])
                .then(|| (PathBuf::from(&message[..at]), message[at + 2..].to_owned()))
        });
        self.entries.push(match borrowed {
            Some((source, message)) => (SiteDiagnostic { source, message }, true),
            None => (
                SiteDiagnostic {
                    source: page.to_path_buf(),
                    message,
                },
                false,
            ),
        });
    }

    /// Every diagnostic a page reports for its own text, then each borrowed diagnostic
    /// once, unless its file reports it itself; in the order they were recorded.
    pub(super) fn finish(self) -> Vec<SiteDiagnostic> {
        let own: BTreeSet<(PathBuf, String)> = self
            .entries
            .iter()
            .filter(|(_, borrowed)| !borrowed)
            .map(|(diagnostic, _)| (diagnostic.source.clone(), diagnostic.message.clone()))
            .collect();
        let mut kept = BTreeSet::new();
        self.entries
            .into_iter()
            .filter(|(diagnostic, borrowed)| {
                let key = (diagnostic.source.clone(), diagnostic.message.clone());
                !borrowed || (!own.contains(&key) && kept.insert(key))
            })
            .map(|(diagnostic, _)| diagnostic)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(diagnostics: Vec<SiteDiagnostic>) -> Vec<(String, String)> {
        diagnostics
            .into_iter()
            .map(|d| (d.source.display().to_string(), d.message))
            .collect()
    }

    #[test]
    fn a_borrowed_diagnostic_moves_to_its_file_and_is_kept_once() {
        let files = [Path::new("pages/a.md"), Path::new("pages/b: c.md")];
        let mut diagnostics = PageDiagnostics::new(files);
        // pages/a.md lists a block of pages/b: c.md twice, and repeats a problem of its own.
        diagnostics.push(files[0], "pages/b: c.md: Missing: X".to_owned());
        diagnostics.push(files[0], "pages/b: c.md: Missing: X".to_owned());
        diagnostics.push(files[0], "pages/b: c.md: Only in a copy".to_owned());
        diagnostics.push(files[0], "Own".to_owned());
        diagnostics.push(files[0], "Own".to_owned());
        diagnostics.push(files[1], "Missing: X".to_owned());
        diagnostics.push(files[1], "pages/elsewhere.md: Kept as written".to_owned());
        assert_eq!(
            pairs(diagnostics.finish()),
            [
                ("pages/b: c.md", "Only in a copy"),
                ("pages/a.md", "Own"),
                ("pages/a.md", "Own"),
                ("pages/b: c.md", "Missing: X"),
                ("pages/b: c.md", "pages/elsewhere.md: Kept as written"),
            ]
            .map(|(source, message)| (source.to_owned(), message.to_owned()))
        );
    }
}
