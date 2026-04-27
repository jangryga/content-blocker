use std::fmt::Display;

use chrono::{DateTime, Local, Utc};
use uuid::Uuid;

pub struct BreakData {
    pub start: DateTime<Local>,
    pub end: Option<DateTime<Local>>,
}

impl BreakData {
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.end.map(|end| end - self.start)
    }
}

pub enum EventType {
    TestEvent,
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::TestEvent => write!(f, "TestEvent"),
        }
    }
}

pub struct SessionEvent {
    pub timestamp: DateTime<Utc>,
    pub id: Uuid,
    pub event_type: EventType,
    pub meta: Box<str>,
}

impl SessionEvent {
    pub fn new(event_type: EventType, meta: impl Into<Box<str>>) -> Self {
        SessionEvent {
            timestamp: Utc::now(),
            id: Uuid::new_v4(),
            event_type,
            meta: meta.into(),
        }
    }
}

pub struct SessionData {
    pub id: Uuid,
    pub start: DateTime<Local>,
    /// The time at which this snapshot was taken. Always `Some` for a collected snapshot.
    pub end: Option<DateTime<Local>>,
    pub breaks: Vec<BreakData>,
}

impl SessionData {
    pub fn total_duration(&self) -> Option<chrono::Duration> {
        self.end.map(|end| end - self.start)
    }

    pub fn break_duration(&self) -> chrono::Duration {
        self.breaks
            .iter()
            .filter_map(|b| b.duration())
            .fold(chrono::Duration::zero(), |acc, d| acc + d)
    }

    pub fn active_duration(&self) -> Option<chrono::Duration> {
        self.total_duration()
            .map(|total| total - self.break_duration())
    }
}
