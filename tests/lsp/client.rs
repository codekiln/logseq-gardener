//! A small Language Server Protocol client for end-to-end tests of `lsg lsp`.
//!
//! [`Garden::copy`] copies the fixture garden `tests/fixtures/lsp-garden` to a temporary
//! folder, and [`Client::start`] starts the built `lsg lsp` on it and completes the
//! `initialize` handshake. A test then sends requests and notifications, waits for what the
//! server sends, and applies a `WorkspaceEdit` to the copy to compare the result with an
//! expected folder.
//!
//! A test file includes this module with `#[path = "lsp/client.rs"] mod client;`.

#![allow(dead_code)]

use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

/// How long a test waits for the server before it fails.
pub const TIMEOUT: Duration = Duration::from_secs(20);

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// The fixture garden in the repository.
pub fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lsp-garden")
}

/// A garden in a temporary folder, removed when the value drops.
pub struct Garden {
    pub root: PathBuf,
}

impl Garden {
    /// A fresh copy of the fixture garden.
    pub fn copy() -> Self {
        Self::copy_of(&fixture())
    }

    /// A fresh copy of any folder.
    pub fn copy_of(source: &Path) -> Self {
        let garden = Self::empty();
        copy_folder(source, &garden.root);
        garden
    }

    /// An empty temporary folder, which is no garden until it holds `logseq/config.edn`.
    pub fn empty() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-lsp-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        // The server and its file watcher report paths with symlinks resolved.
        Self {
            root: root.canonicalize().unwrap(),
        }
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    pub fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path(relative)).unwrap()
    }

    pub fn write(&self, relative: &str, text: &str) {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    pub fn remove(&self, relative: &str) {
        fs::remove_file(self.path(relative)).unwrap();
    }

    /// The `file:` URI of a path relative to the garden root.
    pub fn uri(&self, relative: &str) -> String {
        path_to_uri(&self.path(relative))
    }

    /// Every file beneath the root, by relative path with `/` separators, with its text.
    pub fn files(&self) -> Vec<(String, String)> {
        let mut files = Vec::new();
        list_files(&self.root, &self.root, &mut files);
        files.sort();
        files
    }

    /// Fail unless this garden holds the same files with the same text as `expected`.
    pub fn assert_same_as(&self, expected: &Path) {
        let mut wanted = Vec::new();
        list_files(expected, expected, &mut wanted);
        wanted.sort();
        let actual = self.files();
        let names = |files: &[(String, String)]| -> Vec<String> {
            files.iter().map(|(name, _)| name.clone()).collect()
        };
        assert_eq!(
            names(&actual),
            names(&wanted),
            "the gardens hold different files"
        );
        for ((name, text), (_, wanted)) in actual.iter().zip(&wanted) {
            assert_eq!(text, wanted, "{name} differs");
        }
    }
}

impl Drop for Garden {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn copy_folder(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_folder(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn list_files(root: &Path, folder: &Path, files: &mut Vec<(String, String)>) {
    for entry in fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            list_files(root, &path, files);
        } else {
            let name = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            files.push((name, fs::read_to_string(&path).unwrap()));
        }
    }
}

/// The `file:` URI of an absolute path.
pub fn path_to_uri(path: &Path) -> String {
    let spelling = path.to_string_lossy().replace('\\', "/");
    let mut uri = String::from("file://");
    if !spelling.starts_with('/') {
        uri.push('/');
    }
    for c in spelling.chars() {
        if c.is_ascii_alphanumeric() || "/-._~:".contains(c) {
            uri.push(c);
        } else {
            let mut buffer = [0; 4];
            for unit in c.encode_utf8(&mut buffer).bytes() {
                uri.push_str(&format!("%{unit:02X}"));
            }
        }
    }
    uri
}

/// The absolute path of a `file:` URI the server sent.
pub fn uri_to_path(uri: &str) -> PathBuf {
    let path = uri.strip_prefix("file://").expect("a file URI");
    let raw = path.as_bytes();
    let mut decoded = Vec::new();
    let mut at = 0;
    while at < raw.len() {
        if raw[at] == b'%' {
            let hex = std::str::from_utf8(&raw[at + 1..at + 3]).unwrap();
            decoded.push(u8::from_str_radix(hex, 16).unwrap());
            at += 3;
        } else {
            decoded.push(raw[at]);
            at += 1;
        }
    }
    PathBuf::from(String::from_utf8(decoded).unwrap())
}

/// What the test client tells the server it supports.
#[derive(Clone, Copy)]
pub struct Options {
    /// Offer `utf-8` positions. Without it the server uses UTF-16.
    pub utf8: bool,
    /// Support dynamic registration of file watching, so the test reports file changes
    /// with [`Client::files_changed`]. Without it the server watches the files itself.
    pub client_watches: bool,
    /// Support work done progress.
    pub progress: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            utf8: true,
            client_watches: true,
            progress: true,
        }
    }
}

/// The position of the first `needle` in `text`, in UTF-8 or UTF-16 units, as a protocol
/// `Position`. `inside` moves that many characters into the needle.
pub fn position_of(text: &str, needle: &str, inside: usize, utf8: bool) -> Value {
    let found = text
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} is not in the text"));
    let offset = found
        + needle
            .chars()
            .take(inside)
            .map(char::len_utf8)
            .sum::<usize>();
    position_at(text, offset, utf8)
}

/// The protocol `Position` of a UTF-8 offset in `text`.
pub fn position_at(text: &str, offset: usize, utf8: bool) -> Value {
    let before = &text[..offset];
    let line = before.matches('\n').count();
    let column = &before[before.rfind('\n').map_or(0, |at| at + 1)..];
    let character = if utf8 {
        column.len()
    } else {
        column.encode_utf16().count()
    };
    json!({"line": line, "character": character})
}

/// The UTF-8 offset of a protocol `Position` in `text`.
pub fn offset_of(text: &str, position: &Value, utf8: bool) -> usize {
    let line = position["line"].as_u64().unwrap() as usize;
    let character = position["character"].as_u64().unwrap() as usize;
    let start: usize = text.split_inclusive('\n').take(line).map(str::len).sum();
    let rest = text[start..].split('\n').next().unwrap_or_default();
    if utf8 {
        return start + character.min(rest.len());
    }
    let mut units = 0;
    for (at, c) in rest.char_indices() {
        if units >= character {
            return start + at;
        }
        units += c.len_utf16();
    }
    start + rest.len()
}

/// A running `lsg lsp` and the client's side of the conversation.
pub struct Client {
    child: Child,
    stdin: Option<ChildStdin>,
    incoming: Receiver<Value>,
    next_id: i64,
    /// Offered `utf-8` positions, so positions the client builds use the same unit.
    pub utf8: bool,
    /// The result of `initialize`.
    pub initialize_result: Value,
    /// Notifications the server sent that no call has taken yet, oldest first.
    pub notifications: VecDeque<Value>,
    /// Requests the server sent, each answered with `null`, oldest first.
    pub server_requests: Vec<Value>,
}

impl Client {
    /// Start `lsg lsp` for a garden, with its root as the workspace folder, and complete
    /// the handshake.
    pub fn start(garden: &Garden, options: Options) -> Self {
        let mut client = Self::spawn(options.utf8);
        let result = client
            .initialize(&garden.root, options)
            .unwrap_or_else(|error| panic!("initialize failed: {error}"));
        client.initialize_result = result;
        client.notify("initialized", json!({}));
        client
    }

    /// Start the process without sending anything.
    pub fn spawn(utf8: bool) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_lsg"))
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, incoming) = channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            while let Some(message) = read_message(&mut reader) {
                if sender.send(message).is_err() {
                    break;
                }
            }
        });
        Self {
            stdin: child.stdin.take(),
            child,
            incoming,
            next_id: 0,
            utf8,
            initialize_result: Value::Null,
            notifications: VecDeque::new(),
            server_requests: Vec::new(),
        }
    }

    /// Send `initialize` for a folder and return its result, or the error the server gave.
    pub fn initialize(&mut self, folder: &Path, options: Options) -> Result<Value, Value> {
        let uri = path_to_uri(folder);
        let encodings = if options.utf8 {
            json!(["utf-8", "utf-16"])
        } else {
            json!(["utf-16"])
        };
        self.request_raw(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": uri,
                "workspaceFolders": [{"uri": uri, "name": "garden"}],
                "capabilities": {
                    "general": {"positionEncodings": encodings},
                    "window": {"workDoneProgress": options.progress},
                    "workspace": {
                        "didChangeWatchedFiles": {
                            "dynamicRegistration": options.client_watches,
                            "relativePatternSupport": true,
                        },
                        "workspaceEdit": {
                            "documentChanges": true,
                            "resourceOperations": ["create", "rename", "delete"],
                        },
                    },
                },
            }),
        )
    }

    fn send(&mut self, message: Value) {
        let body = message.to_string();
        let stdin = self.stdin.as_mut().expect("the connection is open");
        write!(stdin, "Content-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        stdin.flush().unwrap();
    }

    /// Send a notification.
    pub fn notify(&mut self, method: &str, params: Value) {
        self.send(json!({"jsonrpc": "2.0", "method": method, "params": params}));
    }

    /// Send a request and return its result. Fails the test when the server answers with
    /// an error.
    pub fn request(&mut self, method: &str, params: Value) -> Value {
        self.request_raw(method, params)
            .unwrap_or_else(|error| panic!("{method} failed: {error}"))
    }

    /// Send a request and return its result, or its error object.
    pub fn request_raw(&mut self, method: &str, params: Value) -> Result<Value, Value> {
        self.next_id += 1;
        let id = self.next_id;
        self.send(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let message = self
                .receive(deadline)
                .unwrap_or_else(|| panic!("no response to {method}"));
            if message.get("method").is_none() && message["id"] == json!(id) {
                return match message.get("error") {
                    Some(error) => Err(error.clone()),
                    None => Ok(message["result"].clone()),
                };
            }
            self.keep(message);
        }
    }

    /// Take one message from the server, or `None` at the deadline or when it has exited.
    fn receive(&mut self, deadline: Instant) -> Option<Value> {
        let wait = deadline.saturating_duration_since(Instant::now());
        self.incoming.recv_timeout(wait).ok()
    }

    /// File a message that is no response to a pending request: answer a server request
    /// with `null`, and queue a notification.
    fn keep(&mut self, message: Value) {
        match (message.get("id"), message.get("method")) {
            (Some(id), Some(_)) => {
                self.send(json!({"jsonrpc": "2.0", "id": id, "result": null}));
                self.server_requests.push(message);
            }
            (None, Some(_)) => self.notifications.push_back(message),
            _ => {}
        }
    }

    /// Take the oldest notification with `method`, waiting for it when none has arrived.
    /// Returns its params.
    pub fn wait_for_notification(&mut self, method: &str) -> Value {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let found = self
                .notifications
                .iter()
                .position(|n| n["method"] == method);
            if let Some(at) = found {
                return self.notifications.remove(at).unwrap()["params"].clone();
            }
            let message = self
                .receive(deadline)
                .unwrap_or_else(|| panic!("the server sent no {method}"));
            self.keep(message);
        }
    }

    /// The oldest request with `method` that the server sent, waiting for it when none has
    /// arrived. Returns its params.
    pub fn wait_for_server_request(&mut self, method: &str) -> Value {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Some(found) = self.server_requests.iter().find(|r| r["method"] == method) {
                return found["params"].clone();
            }
            let message = self
                .receive(deadline)
                .unwrap_or_else(|| panic!("the server sent no {method} request"));
            self.keep(message);
        }
    }

    /// Ask `check` again and again until it returns a value, for state the server reaches
    /// on its own time, such as after a file event.
    pub fn wait_until<T>(
        &mut self,
        what: &str,
        mut check: impl FnMut(&mut Self) -> Option<T>,
    ) -> T {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Some(found) = check(self) {
                return found;
            }
            assert!(Instant::now() < deadline, "timed out waiting until {what}");
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// `textDocument/didOpen` with the given text.
    pub fn open(&mut self, uri: &str, text: &str) {
        self.notify(
            "textDocument/didOpen",
            json!({"textDocument": {"uri": uri, "languageId": "markdown", "version": 1, "text": text}}),
        );
    }

    /// `textDocument/didChange` replacing the first `old` in `text` with `new`. Returns the
    /// text after the change, for the next call.
    pub fn replace(&mut self, uri: &str, version: i64, text: &str, old: &str, new: &str) -> String {
        let start = text
            .find(old)
            .unwrap_or_else(|| panic!("{old:?} is not in the text"));
        let range = json!({
            "start": position_at(text, start, self.utf8),
            "end": position_at(text, start + old.len(), self.utf8),
        });
        self.notify(
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": uri, "version": version},
                "contentChanges": [{"range": range, "text": new}],
            }),
        );
        text.replacen(old, new, 1)
    }

    /// `textDocument/didClose`.
    pub fn close(&mut self, uri: &str) {
        self.notify(
            "textDocument/didClose",
            json!({"textDocument": {"uri": uri}}),
        );
    }

    /// `workspace/didChangeWatchedFiles` for URIs with change types: 1 created, 2 changed,
    /// 3 deleted.
    pub fn files_changed(&mut self, changes: &[(&str, u8)]) {
        let changes: Vec<Value> = changes
            .iter()
            .map(|(uri, kind)| json!({"uri": uri, "type": kind}))
            .collect();
        self.notify(
            "workspace/didChangeWatchedFiles",
            json!({"changes": changes}),
        );
    }

    /// The params of a request that takes a document and a position: the first `needle` in
    /// `text`, `inside` characters in.
    pub fn at(&self, uri: &str, text: &str, needle: &str, inside: usize) -> Value {
        json!({
            "textDocument": {"uri": uri},
            "position": position_of(text, needle, inside, self.utf8),
        })
    }

    /// Apply a `WorkspaceEdit` the server returned to the files on disk: text edits from
    /// `changes` or `documentChanges`, and file creates, renames and deletes.
    pub fn apply_edit(&self, edit: &Value) {
        if let Some(changes) = edit["changes"].as_object() {
            for (uri, edits) in changes {
                self.apply_text_edits(uri, edits.as_array().unwrap());
            }
        }
        for change in edit["documentChanges"].as_array().into_iter().flatten() {
            match change["kind"].as_str() {
                Some("create") => {
                    let path = uri_to_path(change["uri"].as_str().unwrap());
                    fs::create_dir_all(path.parent().unwrap()).unwrap();
                    fs::write(path, "").unwrap();
                }
                Some("rename") => {
                    let to = uri_to_path(change["newUri"].as_str().unwrap());
                    fs::create_dir_all(to.parent().unwrap()).unwrap();
                    fs::rename(uri_to_path(change["oldUri"].as_str().unwrap()), to).unwrap();
                }
                Some("delete") => {
                    fs::remove_file(uri_to_path(change["uri"].as_str().unwrap())).unwrap();
                }
                Some(other) => panic!("unknown document change {other}"),
                None => self.apply_text_edits(
                    change["textDocument"]["uri"].as_str().unwrap(),
                    change["edits"].as_array().unwrap(),
                ),
            }
        }
    }

    fn apply_text_edits(&self, uri: &str, edits: &[Value]) {
        let path = uri_to_path(uri);
        let mut text = fs::read_to_string(&path).unwrap();
        let mut ranges: Vec<(usize, usize, &str)> = edits
            .iter()
            .map(|edit| {
                (
                    offset_of(&text, &edit["range"]["start"], self.utf8),
                    offset_of(&text, &edit["range"]["end"], self.utf8),
                    edit["newText"].as_str().unwrap(),
                )
            })
            .collect();
        // From the end of the file backwards, so earlier offsets stay valid.
        ranges.sort_by_key(|range| std::cmp::Reverse(range.0));
        for (start, end, new_text) in ranges {
            text.replace_range(start..end, new_text);
        }
        fs::write(path, text).unwrap();
    }

    /// Send `shutdown` and `exit` and return the process's exit status.
    pub fn shutdown(mut self) -> ExitStatus {
        self.request("shutdown", Value::Null);
        self.notify("exit", Value::Null);
        self.wait()
    }

    /// Close the connection without `shutdown` and return the exit status and standard
    /// error.
    pub fn disconnect(mut self) -> (ExitStatus, String) {
        self.stdin = None;
        let status = self.wait();
        let mut stderr = String::new();
        self.child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();
        (status, stderr)
    }

    fn wait(&mut self) -> ExitStatus {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status;
            }
            assert!(Instant::now() < deadline, "lsg lsp did not exit");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Read one `Content-Length` framed message, or `None` at the end of the stream.
fn read_message(reader: &mut impl BufRead) -> Option<Value> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length:") {
            length = value.trim().parse::<usize>().ok();
        }
    }
    let mut body = vec![0; length?];
    reader.read_exact(&mut body).ok()?;
    serde_json::from_slice(&body).ok()
}
