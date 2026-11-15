//! One module per feature area. Each handler is registered in [`crate::dispatch`].

pub(crate) mod definition;
pub(crate) mod hover;
pub(crate) mod references;
pub(crate) mod rename;
pub(crate) mod status;
pub(crate) mod sync;
pub(crate) mod watch;
