//! Startup, the main loop and shutdown.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crossbeam_channel::{Receiver, never, select};
use logseq_gardener_sdk::index::{CONFIG_PATH, GardenIndex, IndexedDocument};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{LogMessage, Notification as _, ShowMessage};
use lsp_types::request::{Request as _, Shutdown};
use lsp_types::{
    ClientCapabilities, InitializeParams, InitializeResult, LogMessageParams, MessageType,
    ServerInfo, ShowMessageParams,
};

use crate::capabilities::server_capabilities;
use crate::dispatch;
use crate::features::watch::{self, Watching};
use crate::positions::{Encoding, Paths, uri_to_path};
use crate::progress::Progress;

/// How a session ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Exit {
    /// The client sent `shutdown` and then `exit`.
    Clean,
    /// The session ended another way; the message says how.
    Failed(String),
}

/// One running session: the connection, the index and what the client and server agreed on.
pub(crate) struct Server {
    connection: Connection,
    client: ClientCapabilities,
    pub(crate) index: GardenIndex,
    pub(crate) paths: Paths,
    pub(crate) encoding: Encoding,
    /// The text of each open document, by URI, including documents the garden does not hold.
    pub(crate) buffers: HashMap<String, String>,
    pub(crate) watching: Watching,
    pub(crate) load_milliseconds: f64,
    next_request: i32,
}

impl Server {
    /// The indexed document a `file:` URI names.
    pub(crate) fn document(&self, uri: &str) -> Option<&IndexedDocument> {
        self.index.document(&self.paths.relative_uri(uri)?)
    }

    /// Write a line to the client's log for this server.
    pub(crate) fn log(&self, message: &str) {
        self.notify::<LogMessage>(LogMessageParams {
            typ: MessageType::INFO,
            message: message.to_owned(),
        });
    }

    fn notify<N: lsp_types::notification::Notification>(&self, params: N::Params) {
        let notification = Notification::new(N::METHOD.to_owned(), params);
        let _ = self.connection.sender.send(notification.into());
    }

    fn request_id(&mut self) -> RequestId {
        self.next_request += 1;
        RequestId::from(format!("lsg-{}", self.next_request))
    }

    /// Read the settings and every file, reporting progress under `title`. Open buffers keep
    /// their text. A garden that cannot be read keeps the index it had.
    pub(crate) fn load(&mut self, title: &str) {
        let started = Instant::now();
        let request = self.request_id();
        let mut progress = self.progress(request, title);
        let result = self
            .index
            .reload(&mut |done, total| progress.report(done, total));
        progress.end(format!("{} pages and journals", self.index.len()));
        match result {
            Ok(()) => self.load_milliseconds = milliseconds(started),
            Err(error) => self.log(&format!("cannot read the garden: {error}")),
        }
    }

    fn progress(&self, request: RequestId, title: &str) -> Progress {
        let supported = self
            .client
            .window
            .as_ref()
            .and_then(|window| window.work_done_progress)
            == Some(true);
        let token = format!("lsg/index/{request}");
        Progress::begin(&self.connection.sender, supported, request, token, title)
    }
}

fn milliseconds(started: Instant) -> f64 {
    (started.elapsed().as_secs_f64() * 10_000.0).round() / 10.0
}

/// The nearest directory at or above `start` that holds `logseq/config.edn`.
fn garden_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|folder| folder.join(CONFIG_PATH).is_file())
        .map(Path::to_path_buf)
}

/// The folder the client started the server for: its first workspace folder, else its root
/// URI, else its root path.
#[allow(deprecated)] // `root_uri` and `root_path` are what older clients send.
fn client_folder(params: &InitializeParams) -> Option<PathBuf> {
    params
        .workspace_folders
        .as_ref()
        .and_then(|folders| folders.first())
        .and_then(|folder| uri_to_path(folder.uri.as_str()))
        .or_else(|| uri_to_path(params.root_uri.as_ref()?.as_str()))
        .or_else(|| params.root_path.as_ref().map(PathBuf::from))
}

/// Serve one client on `connection` until it ends the session.
pub fn serve(connection: Connection) -> Exit {
    let (id, params) = match connection.initialize_start() {
        Ok(start) => start,
        Err(error) => {
            return Exit::Failed(format!("the client sent no initialize request: {error}"));
        }
    };
    let params: InitializeParams = match serde_json::from_value(params) {
        Ok(params) => params,
        Err(error) => {
            return refuse(
                &connection,
                id,
                format!("cannot read the initialize request: {error}"),
            );
        }
    };
    let Some(folder) = client_folder(&params) else {
        return refuse(
            &connection,
            id,
            "lsg lsp needs a workspace folder or root URI inside a Logseq garden, a folder that holds logseq/config.edn".to_owned(),
        );
    };
    let Some(root) = garden_root(&folder) else {
        return refuse(
            &connection,
            id,
            format!(
                "{} is not inside a Logseq garden: no folder at or above it holds logseq/config.edn",
                folder.display()
            ),
        );
    };
    let encoding = Encoding::negotiate(
        params
            .capabilities
            .general
            .as_ref()
            .and_then(|general| general.position_encodings.as_deref()),
    );
    let result = InitializeResult {
        capabilities: server_capabilities(encoding),
        server_info: Some(ServerInfo {
            name: "lsg".to_owned(),
            version: Some(crate::VERSION.to_owned()),
        }),
    };
    // Sends the response, then waits for the client's `initialized` notification.
    let result = serde_json::to_value(result).unwrap_or_default();
    if let Err(error) = connection.initialize_finish(id, result) {
        return Exit::Failed(format!("the client did not finish initializing: {error}"));
    }

    // Load the garden. Requests that arrive meanwhile wait in the channel.
    let paths = Paths::new(root);
    let started = Instant::now();
    let supported = params
        .capabilities
        .window
        .as_ref()
        .and_then(|window| window.work_done_progress)
        == Some(true);
    let mut progress = Progress::begin(
        &connection.sender,
        supported,
        RequestId::from("lsg-0".to_owned()),
        "lsg/index/0".to_owned(),
        "Reading the garden",
    );
    let index = match GardenIndex::load_with_progress(paths.root(), &mut |done, total| {
        progress.report(done, total)
    }) {
        Ok(index) => index,
        Err(error) => {
            let message = format!("lsg lsp cannot read the garden: {error}");
            let params = ShowMessageParams {
                typ: MessageType::ERROR,
                message: message.clone(),
            };
            let shown = Notification::new(ShowMessage::METHOD.to_owned(), params);
            let _ = connection.sender.send(shown.into());
            wait_for_the_client_to_leave(&connection);
            return Exit::Failed(message);
        }
    };
    progress.end(format!("{} pages and journals", index.len()));

    let mut server = Server {
        connection,
        client: params.capabilities,
        index,
        paths,
        encoding,
        buffers: HashMap::new(),
        watching: Watching::Nobody,
        load_milliseconds: milliseconds(started),
        next_request: 0,
    };
    let request = server.request_id();
    let sender = server.connection.sender.clone();
    let (watching, problem) = watch::start(
        &server.client,
        &server.paths,
        &mut |request: Request| {
            let _ = sender.send(request.into());
        },
        request,
    );
    server.watching = watching;
    if let Some(problem) = problem {
        server.log(&problem);
    }
    main_loop(&mut server)
}

/// Answer `initialize` with an error the client shows, then wait for the client to go away.
fn refuse(connection: &Connection, id: RequestId, message: String) -> Exit {
    let response = Response::new_err(id, ErrorCode::RequestFailed as i32, message.clone());
    let _ = connection.sender.send(response.into());
    wait_for_the_client_to_leave(connection);
    Exit::Failed(message)
}

/// Answer nothing more until the client sends `exit` or closes the connection, so the
/// message that explains the failure reaches the client before the process ends.
fn wait_for_the_client_to_leave(connection: &Connection) {
    for message in &connection.receiver {
        match message {
            Message::Notification(notification) if notification.method == "exit" => break,
            Message::Request(request) => {
                let response = Response::new_err(
                    request.id,
                    ErrorCode::ServerNotInitialized as i32,
                    "the server did not start".to_owned(),
                );
                let _ = connection.sender.send(response.into());
            }
            _ => {}
        }
    }
}

/// After `shutdown`: wait for `exit`. The client's answers to the server's own requests and
/// its late notifications can still arrive first, and they need no reply. A request gets an
/// error, as the protocol asks.
fn wait_for_exit(connection: &Connection) -> Exit {
    for message in &connection.receiver {
        match message {
            Message::Notification(notification) if notification.method == "exit" => {
                return Exit::Clean;
            }
            Message::Request(request) => {
                let response = Response::new_err(
                    request.id,
                    ErrorCode::InvalidRequest as i32,
                    "the server is shutting down".to_owned(),
                );
                let _ = connection.sender.send(response.into());
            }
            _ => {}
        }
    }
    Exit::Failed("the client closed the connection after shutdown without exit".to_owned())
}

fn main_loop(server: &mut Server) -> Exit {
    loop {
        let events: Receiver<notify::Result<notify::Event>> = match &server.watching {
            Watching::Server { events, .. } => events.clone(),
            _ => never(),
        };
        let message = select! {
            recv(server.connection.receiver) -> message => match message {
                Ok(message) => message,
                Err(_) => return Exit::Failed("the client closed the connection without shutdown".to_owned()),
            },
            recv(events) -> event => {
                if let Ok(event) = event {
                    watch::server_event(server, event);
                }
                continue;
            }
        };
        match message {
            Message::Request(request) => {
                if request.method == Shutdown::METHOD {
                    let answer = Response::new_ok(request.id, serde_json::Value::Null);
                    let _ = server.connection.sender.send(answer.into());
                    return wait_for_exit(&server.connection);
                }
                let Request { id, method, params } = request;
                let response = match dispatch::request(server, &method, params) {
                    Some(Ok(result)) => Response::new_ok(id, result),
                    Some(Err(error)) => Response::new_err(id, error.code, error.message),
                    None => Response::new_err(
                        id,
                        ErrorCode::MethodNotFound as i32,
                        format!("lsg lsp does not answer {method}"),
                    ),
                };
                let _ = server.connection.sender.send(response.into());
            }
            Message::Notification(notification) => {
                if notification.method == "exit" {
                    return Exit::Failed("the client sent exit without shutdown".to_owned());
                }
                dispatch::notification(server, &notification.method, notification.params);
            }
            // Answers to the server's own requests, which need no follow-up.
            Message::Response(_) => {}
        }
    }
}
