use chrono::Local;
use std::cell::Cell;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs, io, time};
use uuid::Uuid;

use super::Session;
use super::data::{BreakData, SessionData};
use super::formatter::SessionFormatter;

struct BreakRecord {
    start: time::Instant,
    end: Cell<Option<time::Instant>>,
}

pub struct LocalSession {
    id: Uuid,
    log_file: fs::File,
    start_instant: time::Instant,
    start_wall: chrono::DateTime<Local>,
    breaks: Vec<BreakRecord>,
    is_running: AtomicBool,
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

        let log_file = fs::File::options()
            .append(true)
            .create(true)
            .open(log_dir.join("web-blocker.log"))?;

        Ok(LocalSession {
            id,
            log_file,
            start_instant: time::Instant::now(),
            start_wall: Local::now(),
            is_running: AtomicBool::new(true),
            breaks: Vec::new(),
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

    fn save(&mut self, formatter: &dyn SessionFormatter) -> io::Result<()> {
        let content = formatter.format(&self.collect());
        self.log_file.write_all(content.as_bytes())
    }
}
