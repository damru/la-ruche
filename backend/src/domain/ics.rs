// ── domain/ics.rs — Export ICS du calendrier (EP-014-S04) ─────────────────
//
// Minimal RFC 5545 serializer: VCALENDAR wrapping one VEVENT per
// CalendarEvent (+ VALARM for deadlines). Pure, no I/O.

use chrono::Utc;

use crate::domain::calendar::CalendarEvent;

/// RFC 5545 line folding: lines must not exceed 75 octets: continuation
/// lines start with a single space, joined by CRLF.
fn fold_line(line: &str) -> String {
    let bytes = line.as_bytes();
    if bytes.len() <= 75 {
        return line.to_string();
    }

    let mut folded = String::new();
    let mut start = 0;
    let mut first = true;
    while start < bytes.len() {
        let limit = if first { 75 } else { 74 };
        let mut end = (start + limit).min(bytes.len());
        // Don't split a UTF-8 char in the middle.
        while end < bytes.len() && !line.is_char_boundary(end) {
            end -= 1;
        }
        if !first {
            folded.push_str("\r\n ");
        }
        folded.push_str(&line[start..end]);
        first = false;
        start = end;
    }
    folded
}

/// Escapes a text value per RFC 5545 §3.3.11.
fn escape_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(',', "\\,")
        .replace(';', "\\;")
        .replace('\n', "\\n")
}

fn event_to_vevent(event: &CalendarEvent, base_url: &str, dtstamp: &str) -> String {
    let mut lines = vec!["BEGIN:VEVENT".to_string()];
    lines.push(format!("UID:{}@laruche", event.id));
    lines.push(format!("DTSTAMP:{dtstamp}"));

    match event.time {
        Some(time) => {
            let dt = event.date.and_time(time);
            lines.push(format!("DTSTART:{}Z", dt.format("%Y%m%dT%H%M%S")));
        }
        None => {
            lines.push(format!(
                "DTSTART;VALUE=DATE:{}",
                event.date.format("%Y%m%d")
            ));
        }
    }

    lines.push(format!("SUMMARY:{}", escape_text(&event.title)));
    let url = format!("{base_url}{}", event.link);
    lines.push(format!("DESCRIPTION:{}", escape_text(&url)));
    lines.push(format!("URL:{}", escape_text(&url)));
    lines.push(format!(
        "CATEGORIES:{}",
        escape_text(&format!("{:?}", event.source))
    ));

    if event.is_deadline {
        lines.push("BEGIN:VALARM".to_string());
        lines.push("ACTION:DISPLAY".to_string());
        lines.push("TRIGGER:-PT24H".to_string());
        lines.push(format!("DESCRIPTION:{}", escape_text(&event.title)));
        lines.push("END:VALARM".to_string());
    }

    lines.push("END:VEVENT".to_string());

    lines
        .into_iter()
        .map(|l| fold_line(&l))
        .collect::<Vec<_>>()
        .join("\r\n")
}

/// RFC 5545 minimal serializer: VCALENDAR wrapping one VEVENT per
/// CalendarEvent. `base_url` prefixes each event's relative `link` to
/// build an absolute `URL` (AC-CAL-003-C).
pub fn to_ics(events: &[CalendarEvent], prodid: &str, base_url: &str) -> String {
    let dtstamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

    let mut lines = vec![
        "BEGIN:VCALENDAR".to_string(),
        "VERSION:2.0".to_string(),
        fold_line(&format!("PRODID:{}", escape_text(prodid))),
        "CALSCALE:GREGORIAN".to_string(),
    ];

    for event in events {
        lines.push(event_to_vevent(event, base_url, &dtstamp));
    }

    lines.push("END:VCALENDAR".to_string());
    lines.join("\r\n") + "\r\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::calendar::CalendarSource;
    use chrono::{NaiveDate, NaiveTime};

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    fn base_event() -> CalendarEvent {
        CalendarEvent {
            id: "meeting:abc".to_string(),
            source: CalendarSource::Meeting,
            title: "Assemblée générale".to_string(),
            date: date(2026, 3, 15),
            time: None,
            link: "/meetings/abc".to_string(),
            is_deadline: false,
        }
    }

    #[test]
    fn simple_dated_event_has_uid_dtstart_summary_url() {
        let ics = to_ics(&[base_event()], "-//La Ruche//FR", "https://app.example");
        assert!(ics.contains("BEGIN:VCALENDAR"));
        assert!(ics.contains("END:VCALENDAR"));
        assert!(ics.contains("UID:meeting:abc@laruche"));
        assert!(ics.contains("DTSTART;VALUE=DATE:20260315"));
        assert!(ics.contains("SUMMARY:Assemblée générale"));
        assert!(ics.contains("URL:https://app.example/meetings/abc"));
        assert!(!ics.contains("VALARM"));
    }

    #[test]
    fn timed_event_uses_utc_dtstart() {
        let mut event = base_event();
        event.time = Some(NaiveTime::from_hms_opt(18, 30, 0).unwrap());
        let ics = to_ics(&[event], "-//La Ruche//FR", "https://app.example");
        assert!(ics.contains("DTSTART:20260315T183000Z"));
        assert!(!ics.contains("VALUE=DATE"));
    }

    #[test]
    fn deadline_event_has_valarm_with_trigger() {
        let mut event = base_event();
        event.is_deadline = true;
        let ics = to_ics(&[event], "-//La Ruche//FR", "https://app.example");
        assert!(ics.contains("BEGIN:VALARM"));
        assert!(ics.contains("ACTION:DISPLAY"));
        assert!(ics.contains("TRIGGER:-PT24H"));
        assert!(ics.contains("END:VALARM"));
    }

    #[test]
    fn non_deadline_event_has_no_valarm() {
        let event = base_event();
        let ics = to_ics(&[event], "-//La Ruche//FR", "https://app.example");
        assert!(!ics.contains("VALARM"));
    }

    #[test]
    fn special_characters_are_escaped() {
        let mut event = base_event();
        event.title = "Réunion, urgente; contrat\nrenouvelé".to_string();
        let ics = to_ics(&[event], "-//La Ruche//FR", "https://app.example");
        assert!(ics.contains("Réunion\\, urgente\\; contrat\\nrenouvelé"));
    }

    #[test]
    fn long_line_is_folded_at_75_octets() {
        let mut event = base_event();
        event.title = "A".repeat(120);
        let ics = to_ics(&[event], "-//La Ruche//FR", "https://app.example");
        // Folded continuation lines are prefixed by CRLF + space.
        assert!(ics.contains("\r\n "));
        for line in ics.split("\r\n") {
            assert!(line.len() <= 75, "line too long: {line:?}");
        }
    }

    #[test]
    fn empty_events_produce_valid_empty_calendar() {
        let ics = to_ics(&[], "-//La Ruche//FR", "https://app.example");
        assert!(ics.contains("BEGIN:VCALENDAR"));
        assert!(ics.contains("END:VCALENDAR"));
        assert!(!ics.contains("BEGIN:VEVENT"));
    }
}
