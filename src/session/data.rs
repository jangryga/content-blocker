use chrono::{DateTime, Local};
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
