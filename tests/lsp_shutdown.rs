//! End-to-end tests of how `lsg lsp` ends a session.

#[path = "lsp/client.rs"]
mod client;

use client::{Client, Garden, Options};
use serde_json::Value;

#[test]
fn shutdown_right_after_initialized_exits_cleanly() {
    let garden = Garden::copy();
    // The client watches files, so the server sends `client/registerCapability` and a
    // progress token request while it loads. The client answers them after it has sent
    // `shutdown`, and those answers reach the server before `exit` does.
    let client = Client::start(&garden, Options::default());
    assert_eq!(client.shutdown().code(), Some(0));
}

#[test]
fn a_request_after_shutdown_gets_an_error_and_exit_still_ends_the_session() {
    let garden = Garden::copy();
    let mut client = Client::start(&garden, Options::default());
    client.request("shutdown", Value::Null);
    let error = client.request_raw("lsg/status", Value::Null).unwrap_err();
    // InvalidRequest.
    assert_eq!(error["code"], -32600);
    client.notify("exit", Value::Null);
    let (status, _) = client.disconnect();
    assert_eq!(status.code(), Some(0));
}
