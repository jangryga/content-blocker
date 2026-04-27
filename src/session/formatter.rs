use super::data::{SessionData, SessionEvent};

pub trait SessionFormatter: Send + Sync {
    fn format(&self, data: &SessionData) -> String;
    fn format_event(&self, event: &SessionEvent) -> String;
}

pub struct PrettyFormatter;

fn fmt_duration(d: chrono::Duration) -> String {
    let total = d.num_seconds().max(0) as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}

impl SessionFormatter for PrettyFormatter {
    fn format(&self, data: &SessionData) -> String {
        let mut breaks_section = String::new();
        for (i, b) in data.breaks.iter().enumerate() {
            let end_str = b
                .end
                .map(|e| e.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "ongoing".to_string());
            let dur_str = b
                .duration()
                .map(fmt_duration)
                .unwrap_or_else(|| "—".to_string());
            breaks_section.push_str(&format!(
                "  #{:<2} {}  →  {}  ({})\n",
                i + 1,
                b.start.format("%H:%M:%S"),
                end_str,
                dur_str,
            ));
        }

        let end_str = data
            .end
            .map(|e| e.format("%Y-%m-%d %H:%M:%S %Z").to_string())
            .unwrap_or_else(|| "ongoing".to_string());

        format!(
            "=== web-blocker session ===\n\
             ID:          {}\n\
             Started:     {}\n\
             Ended:       {}\n\
             Duration:    {}\n\
             \n\
             Breaks ({}):\n\
             {}\
             Active time: {}\n",
            data.id,
            data.start.format("%Y-%m-%d %H:%M:%S %Z"),
            end_str,
            data.total_duration()
                .map(fmt_duration)
                .unwrap_or_else(|| "ongoing".to_string()),
            data.breaks.len(),
            if data.breaks.is_empty() {
                "  (none)\n".to_string()
            } else {
                breaks_section
            },
            data.active_duration()
                .map(fmt_duration)
                .unwrap_or_else(|| "—".to_string()),
        )
    }

    fn format_event(&self, event: &SessionEvent) -> String {
        format!(
            "{} {}[{}] {}\n",
            event.timestamp, event.event_type, event.id, event.meta
        )
    }
}

pub struct JsonFormatter;

fn opt_quoted(v: Option<impl std::fmt::Display>) -> String {
    match v {
        Some(s) => format!("\"{}\"", s),
        None => "null".to_string(),
    }
}

impl SessionFormatter for JsonFormatter {
    fn format(&self, data: &SessionData) -> String {
        let breaks: Vec<String> = data
            .breaks
            .iter()
            .map(|b| {
                format!(
                    "{{\"start\":\"{}\",\"end\":{},\"duration_secs\":{}}}",
                    b.start.to_rfc3339(),
                    opt_quoted(b.end.map(|e| e.to_rfc3339())),
                    b.duration()
                        .map(|d| d.num_seconds().to_string())
                        .unwrap_or_else(|| "null".to_string()),
                )
            })
            .collect();

        format!(
            "{{\
                \"id\":\"{}\",\
                \"started\":\"{}\",\
                \"ended\":{},\
                \"duration_secs\":{},\
                \"breaks\":[{}],\
                \"active_secs\":{}\
            }}",
            data.id,
            data.start.to_rfc3339(),
            opt_quoted(data.end.map(|e| e.to_rfc3339())),
            data.total_duration()
                .map(|d| d.num_seconds().to_string())
                .unwrap_or_else(|| "null".to_string()),
            breaks.join(","),
            data.active_duration()
                .map(|d| d.num_seconds().to_string())
                .unwrap_or_else(|| "null".to_string()),
        )
    }

    fn format_event(&self, _event: &SessionEvent) -> String {
        todo!("SessionFormatter::format_event")
    }
}
