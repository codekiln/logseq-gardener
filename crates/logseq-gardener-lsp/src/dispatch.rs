//! Which function answers each request and notification.
//!
//! A feature adds one line to [`request`] or [`notification`]. Each handler takes the server
//! and its typed parameters; [`run`] and [`notify`] decode the parameters and encode the
//! result.

use lsp_server::{ErrorCode, ResponseError};
use lsp_types::notification::{
    DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument,
    Notification,
};
use lsp_types::request::{Completion, GotoDefinition, HoverRequest, References, Request};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::features::{completion, definition, hover, references, status, sync, watch};
use crate::server::Server;

/// What a request handler returns.
pub(crate) type Answer<T> = Result<T, ResponseError>;

/// Answer a request, or `None` when no handler takes the method.
pub(crate) fn request(server: &mut Server, method: &str, params: Value) -> Option<Answer<Value>> {
    Some(match method {
        Completion::METHOD => run(server, params, completion::completion),
        GotoDefinition::METHOD => run(server, params, definition::definition),
        References::METHOD => run(server, params, references::references),
        HoverRequest::METHOD => run(server, params, hover::hover),
        status::STATUS => run(server, params, status::status),
        status::OCCURRENCE => run(server, params, status::occurrence),
        _ => return None,
    })
}

/// Handle a notification. Returns whether a handler took the method.
pub(crate) fn notification(server: &mut Server, method: &str, params: Value) -> bool {
    match method {
        DidOpenTextDocument::METHOD => notify(server, params, sync::did_open),
        DidChangeTextDocument::METHOD => notify(server, params, sync::did_change),
        DidCloseTextDocument::METHOD => notify(server, params, sync::did_close),
        DidChangeWatchedFiles::METHOD => notify(server, params, watch::did_change_watched_files),
        _ => return false,
    }
    true
}

/// An error response for parameters or state the server cannot use.
pub(crate) fn invalid(message: impl Into<String>) -> ResponseError {
    ResponseError {
        code: ErrorCode::InvalidParams as i32,
        message: message.into(),
        data: None,
    }
}

fn run<P: DeserializeOwned, R: Serialize>(
    server: &mut Server,
    params: Value,
    handler: fn(&mut Server, P) -> Answer<R>,
) -> Answer<Value> {
    let params = serde_json::from_value(params).map_err(|error| invalid(error.to_string()))?;
    let result = handler(server, params)?;
    serde_json::to_value(result).map_err(|error| ResponseError {
        code: ErrorCode::InternalError as i32,
        message: error.to_string(),
        data: None,
    })
}

fn notify<P: DeserializeOwned>(server: &mut Server, params: Value, handler: fn(&mut Server, P)) {
    match serde_json::from_value(params) {
        Ok(params) => handler(server, params),
        Err(error) => server.log(&format!("ignored a notification: {error}")),
    }
}
