use std::io;

use super::Session;
use super::data::SessionData;
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
}
