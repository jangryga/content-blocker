use std::io;

use super::Session;
use super::data::{SessionData, SessionEvent};
use super::formatter::SessionFormatter;

pub struct WebSession;

impl Session for WebSession {
    fn start(&mut self) -> io::Result<()> {
        todo!("WebSession::start")
    }

    fn stop(&mut self) -> io::Result<()> {
        todo!("WebSession::stop")
    }

    fn collect(&self) -> SessionData {
        todo!("WebSession::collect")
    }

    fn save(&mut self, _formatter: &dyn SessionFormatter) -> io::Result<()> {
        todo!("WebSession::save")
    }

    fn drain(&mut self, _formatter: &dyn SessionFormatter) -> io::Result<()> {
        todo!("WebSession::drain")
    }

    fn log_event(
        &mut self,
        _event: SessionEvent,
        _formatter: &dyn SessionFormatter,
    ) -> io::Result<()> {
        todo!("WebSession::log_event")
    }
}
