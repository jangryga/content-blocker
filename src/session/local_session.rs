use chrono::Local;
use std::cell::Cell;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs, io, time};
use uuid::Uuid;

use super::Session;
use super::data::{BreakData, SessionData, SessionEvent};
use super::formatter::SessionFormatter;

struct BreakRecord {
    start: time::Instant,
    end: Cell<Option<time::Instant>>,
}

pub struct LocalSession {
    id: Uuid,
    session_log_file: fs::File,
    event_log_file: fs::File,
    start_instant: time::Instant,
    start_wall: chrono::DateTime<Local>,
    breaks: Vec<BreakRecord>,
    is_running: AtomicBool,
    event_sink: Vec<SessionEvent>,
    event_sink_max_size: usize,
}

impl LocalSession {
    pub fn new() -> io::Result<Self> {
        let id = Uuid::new_v4();

        let log_dir = dirs::cache_dir()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "could not resolve home directory")
            })?
            .join("../Logs/web-blocker");

        fs::create_dir_all(&log_dir)?;

        let session_log_file = fs::File::options()
            .append(true)
            .create(true)
            .open(log_dir.join("sessions.log"))?;

        let event_log_file = fs::File::options()
            .append(true)
            .create(true)
            .open(log_dir.join("events.log"))?;

        Ok(LocalSession {
            id,
            session_log_file,
            event_log_file,
            start_instant: time::Instant::now(),
            start_wall: Local::now(),
            is_running: AtomicBool::new(true),
            breaks: Vec::new(),
            event_sink: Vec::new(),
            event_sink_max_size: 1,
        })
    }

    /// Converts a monotonic `Instant` to a wall-clock `DateTime` relative to
    /// the session start, avoiding the need to store a `SystemTime` per event.
    fn to_wall(&self, instant: time::Instant) -> chrono::DateTime<Local> {
        let offset = instant.duration_since(self.start_instant);
        self.start_wall + chrono::Duration::from_std(offset).unwrap_or_default()
    }
}

impl Session for LocalSession {
    fn start(&mut self) -> io::Result<()> {
        if self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }
        if let Some(last) = self.breaks.last() {
            last.end.set(Some(time::Instant::now()));
        }
        self.is_running.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn stop(&mut self) -> io::Result<()> {
        if !self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }
        self.breaks.push(BreakRecord {
            start: time::Instant::now(),
            end: Cell::new(None),
        });
        self.is_running.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn collect(&self) -> SessionData {
        SessionData {
            id: self.id,
            start: self.start_wall,
            end: Some(self.to_wall(time::Instant::now())),
            breaks: self
                .breaks
                .iter()
                .map(|b| BreakData {
                    start: self.to_wall(b.start),
                    end: b.end.get().map(|e| self.to_wall(e)),
                })
                .collect(),
        }
    }

    fn log_event(
        &mut self,
        event: SessionEvent,
        formatter: &dyn SessionFormatter,
    ) -> io::Result<()> {
        self.event_sink.push(event);
        if self.event_sink.len() == self.event_sink_max_size {
            self.drain(formatter)?;
        }
        self.event_sink = Vec::new();
        Ok(())
    }

    fn drain(&mut self, formatter: &dyn SessionFormatter) -> io::Result<()> {
        for event in self.event_sink.iter_mut() {
            let log = formatter.format_event(&event);
            self.event_log_file.write_all(log.as_bytes())?
        }
        Ok(())
    }

    fn save(&mut self, formatter: &dyn SessionFormatter) -> io::Result<()> {
        self.stop()?;
        let content = formatter.format(&self.collect());
        self.session_log_file.write_all(content.as_bytes())
    }
}
