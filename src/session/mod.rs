mod local_session;
mod web_session;

pub mod data;
pub mod formatter;

pub use data::{BreakData, SessionData};
pub use formatter::{JsonFormatter, PrettyFormatter, SessionFormatter};
pub use local_session::LocalSession;
pub use web_session::WebSession;

use std::io;
use std::sync::{Mutex, OnceLock};

pub trait Session: Send {
    fn start(&mut self) -> io::Result<()>;
    fn stop(&mut self) -> io::Result<()>;
    /// Collects a point-in-time snapshot of session state as a plain data type.
    /// All formatters operate on this; implementations never need to know about formatting.
    fn collect(&self) -> SessionData;
    /// Formats the current snapshot with `formatter` and persists it.
    fn save(&mut self, formatter: &dyn SessionFormatter) -> io::Result<()>;
}

static INSTANCE: OnceLock<Mutex<Box<dyn Session + Send>>> = OnceLock::new();

/// Initialises the global session. Must be called once before any other functions.
/// Subsequent calls are silently ignored.
pub fn init(session: Box<dyn Session + Send>) {
    let _ = INSTANCE.set(Mutex::new(session));
}

fn with_session<F, R>(f: F) -> io::Result<R>
where
    F: FnOnce(&mut dyn Session) -> io::Result<R>,
{
    let mutex = INSTANCE
        .get()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "session not initialised"))?;
    let mut guard = mutex
        .lock()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "session lock poisoned"))?;
    f(guard.as_mut())
}

pub fn start() -> io::Result<()> {
    with_session(|s| s.start())
}

pub fn stop() -> io::Result<()> {
    with_session(|s| s.stop())
}

pub fn save(formatter: &dyn SessionFormatter) -> io::Result<()> {
    with_session(|s| s.save(formatter))
}
