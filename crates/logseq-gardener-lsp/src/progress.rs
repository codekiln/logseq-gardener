//! Work done progress for loading the garden.

use crossbeam_channel::Sender;
use lsp_server::{Message, Notification, Request, RequestId};
use lsp_types::notification::{Notification as _, Progress as ProgressNotification};
use lsp_types::request::{Request as _, WorkDoneProgressCreate};
use lsp_types::{
    NumberOrString, ProgressParams, ProgressParamsValue, WorkDoneProgress, WorkDoneProgressBegin,
    WorkDoneProgressCreateParams, WorkDoneProgressEnd, WorkDoneProgressReport,
};

/// Reports one piece of work to a client that supports work done progress, and nothing to
/// one that does not.
pub(crate) struct Progress {
    sender: Option<Sender<Message>>,
    token: NumberOrString,
    percentage: u32,
}

impl Progress {
    /// Create a progress token on the client and report that the work began. `request` is
    /// the id for the `window/workDoneProgress/create` request.
    pub(crate) fn begin(
        sender: &Sender<Message>,
        supported: bool,
        request: RequestId,
        token: String,
        title: &str,
    ) -> Self {
        let progress = Self {
            sender: supported.then(|| sender.clone()),
            token: NumberOrString::String(token),
            percentage: 0,
        };
        if let Some(sender) = &progress.sender {
            let params = WorkDoneProgressCreateParams {
                token: progress.token.clone(),
            };
            let create = Request::new(request, WorkDoneProgressCreate::METHOD.to_owned(), params);
            let _ = sender.send(create.into());
        }
        progress.send(WorkDoneProgress::Begin(WorkDoneProgressBegin {
            title: title.to_owned(),
            cancellable: Some(false),
            message: None,
            percentage: Some(0),
        }));
        progress
    }

    /// Report that `done` of `total` steps are finished, when the whole-number percentage
    /// moved by five or more since the last report.
    pub(crate) fn report(&mut self, done: usize, total: usize) {
        let percentage = (done * 100).checked_div(total).unwrap_or(100).min(100) as u32;
        if percentage < self.percentage + 5 {
            return;
        }
        self.percentage = percentage;
        self.send(WorkDoneProgress::Report(WorkDoneProgressReport {
            cancellable: Some(false),
            message: Some(format!("{done} of {total} files")),
            percentage: Some(percentage),
        }));
    }

    /// Report that the work ended.
    pub(crate) fn end(self, message: String) {
        self.send(WorkDoneProgress::End(WorkDoneProgressEnd {
            message: Some(message),
        }));
    }

    fn send(&self, value: WorkDoneProgress) {
        let Some(sender) = &self.sender else {
            return;
        };
        let params = ProgressParams {
            token: self.token.clone(),
            value: ProgressParamsValue::WorkDone(value),
        };
        let notification = Notification::new(ProgressNotification::METHOD.to_owned(), params);
        let _ = sender.send(notification.into());
    }
}
