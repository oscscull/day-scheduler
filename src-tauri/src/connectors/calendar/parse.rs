//! Turns an iCal feed into the event occurrences within a time window, expanding recurring
//! events and applying their exceptions, moved instances and cancellations.

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use ical::IcalParser;
use ical::parser::ical::component::IcalEvent;
use ical::property::Property;
use rrule::{RRuleSet, Tz};

/// One time an event happens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    /// Stable across syncs: the event's UID plus this occurrence's start.
    pub id: String,
    pub title: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub location: Option<String>,
    pub join_url: Option<String>,
    pub all_day: bool,
}

const MAX_PER_EVENT: u16 = 500;

fn prop<'a>(event: &'a IcalEvent, name: &str) -> Option<&'a Property> {
    event.properties.iter().find(|p| p.name == name)
}

fn value<'a>(event: &'a IcalEvent, name: &str) -> Option<&'a str> {
    prop(event, name).and_then(|p| p.value.as_deref())
}

/// A property back in iCal form, e.g. `DTSTART;TZID=Europe/London:20261021T093000`.
fn line(p: &Property, keep_tzid: bool) -> String {
    let params = p
        .params
        .iter()
        .flatten()
        .filter(|(k, _)| keep_tzid || k != "TZID")
        .fold(String::new(), |mut acc, (k, v)| {
            acc.push(';');
            acc.push_str(k);
            acc.push('=');
            acc.push_str(&v.join(","));
            acc
        });
    format!(
        "{}{params}:{}",
        p.name,
        p.value.as_deref().unwrap_or_default()
    )
}

/// Video call hosts whose links make a Join button.
const CALL_HOSTS: [&str; 4] = [
    "meet.google.com",
    "zoom.us",
    "teams.microsoft.com",
    "whereby.com",
];

fn first_call_link(text: &str) -> Option<String> {
    text.split(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '\\'))
        .filter(|w| w.starts_with("https://"))
        .find(|w| CALL_HOSTS.iter().any(|h| w.contains(h)))
        .map(|w| w.trim_end_matches(['.', ',', ')']).to_owned())
}

/// The event's video call link, from conference fields first, then the location and notes.
fn join_url(event: &IcalEvent) -> Option<String> {
    ["X-GOOGLE-CONFERENCE", "CONFERENCE"]
        .iter()
        .find_map(|name| {
            value(event, name)
                .filter(|v| v.starts_with("https://"))
                .map(str::to_owned)
        })
        .or_else(|| {
            ["LOCATION", "DESCRIPTION", "URL"]
                .iter()
                .find_map(|name| value(event, name).and_then(first_call_link))
        })
}

fn is_all_day(p: &Property) -> bool {
    p.params
        .iter()
        .flatten()
        .any(|(k, v)| k == "VALUE" && v.iter().any(|x| x == "DATE"))
        || p.value.as_deref().is_some_and(|v| v.len() == 8)
}

/// Start times in the window. Unknown time zone names fall back to local time.
fn starts(rule_text: &[String], from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<DateTime<Utc>> {
    let expand = |text: String| -> Option<Vec<DateTime<Utc>>> {
        let set: RRuleSet = text.parse().ok()?;
        let dates = set
            .after(from.with_timezone(&Tz::UTC))
            .before(to.with_timezone(&Tz::UTC))
            .all(MAX_PER_EVENT)
            .dates;
        Some(dates.into_iter().map(|d| d.with_timezone(&Utc)).collect())
    };
    rule_text.first().map_or_else(Vec::new, |first| {
        expand(rule_text.join("\n"))
            .or_else(|| {
                expand(
                    std::iter::once(first.replace(";TZID=", ";X-TZID="))
                        .chain(rule_text.iter().skip(1).cloned())
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            })
            .unwrap_or_default()
    })
}

/// The first start of a single property, e.g. DTEND, for working out durations.
fn first_time(p: &Property) -> Option<DateTime<Utc>> {
    let text = |keep| {
        format!(
            "{}\nRRULE:FREQ=DAILY;COUNT=1",
            line(p, keep).replacen(&p.name, "DTSTART", 1)
        )
    };
    let parse = |t: String| {
        t.parse::<RRuleSet>()
            .ok()
            .and_then(|s| s.all(1).dates.into_iter().next())
    };
    parse(text(true))
        .or_else(|| parse(text(false)))
        .map(|d| d.with_timezone(&Utc))
}

fn duration(event: &IcalEvent, dtstart: &Property, all_day: bool) -> Duration {
    let start = first_time(dtstart);
    let end = prop(event, "DTEND").and_then(first_time);
    match (start, end) {
        (Some(s), Some(e)) if e > s => e - s,
        _ if all_day => Duration::days(1),
        _ => Duration::minutes(30),
    }
}

/// UIDs mapped to the RECURRENCE-ID lines of their moved or cancelled instances.
fn overrides(events: &[IcalEvent]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for e in events {
        if let (Some(uid), Some(rid)) = (value(e, "UID"), prop(e, "RECURRENCE-ID")) {
            map.entry(uid.to_owned())
                .or_default()
                .push(line(rid, true).replacen("RECURRENCE-ID", "EXDATE", 1));
        }
    }
    map
}

fn rule_text(event: &IcalEvent, dtstart: &Property, exdates: &[String]) -> Vec<String> {
    let mut lines = vec![line(dtstart, true)];
    let recurring = prop(event, "RECURRENCE-ID").is_none();
    match value(event, "RRULE").filter(|_| recurring) {
        Some(rule) => {
            lines.push(format!("RRULE:{rule}"));
            lines.extend(
                event
                    .properties
                    .iter()
                    .filter(|p| p.name == "EXDATE")
                    .map(|p| line(p, true)),
            );
            lines.extend(exdates.iter().cloned());
        }
        None => lines.push("RRULE:FREQ=DAILY;COUNT=1".to_owned()),
    }
    lines
}

fn expand(
    event: &IcalEvent,
    exdates: &[String],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Vec<Occurrence> {
    let (Some(uid), Some(dtstart)) = (value(event, "UID"), prop(event, "DTSTART")) else {
        return Vec::new();
    };
    if value(event, "STATUS") == Some("CANCELLED") {
        return Vec::new();
    }
    let all_day = is_all_day(dtstart);
    let length = duration(event, dtstart, all_day);
    let title = value(event, "SUMMARY").unwrap_or_default().to_owned();
    let join = join_url(event);
    let location = value(event, "LOCATION")
        .filter(|l| !l.is_empty())
        .map(str::to_owned);
    starts(&rule_text(event, dtstart, exdates), from - length, to)
        .into_iter()
        .map(|start| Occurrence {
            id: format!("{uid}@{}", start.to_rfc3339()),
            title: title.clone(),
            start,
            end: start + length,
            location: location.clone(),
            join_url: join.clone(),
            all_day,
        })
        .collect()
}

/// Every occurrence that overlaps `[from, to)`.
pub fn occurrences(ics: &str, from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<Occurrence> {
    let events: Vec<IcalEvent> = IcalParser::new(ics.as_bytes())
        .filter_map(Result::ok)
        .flat_map(|c| c.events)
        .collect();
    let moved = overrides(&events);
    events
        .iter()
        .flat_map(|e| {
            let exdates = value(e, "UID")
                .and_then(|uid| moved.get(uid))
                .map_or(&[][..], Vec::as_slice);
            expand(e, exdates, from, to)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;

    const FEED: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n\
BEGIN:VEVENT\r\nUID:standup\r\nSUMMARY:Standup\r\nDTSTART;TZID=Europe/London:20261019T093000\r\n\
DTEND;TZID=Europe/London:20261019T094500\r\nRRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR\r\n\
EXDATE;TZID=Europe/London:20261022T093000\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:standup\r\nRECURRENCE-ID;TZID=Europe/London:20261021T093000\r\nSUMMARY:Standup (moved)\r\n\
DTSTART;TZID=Europe/London:20261021T100000\r\nDTEND;TZID=Europe/London:20261021T101500\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:dentist\r\nSUMMARY:Dentist\r\nLOCATION:Smile Dental\r\nDESCRIPTION:Join: https://meet.google.com/abc-defg-hij.\r\nDTSTART:20261021T171500Z\r\n\
DTEND:20261021T180000Z\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:holiday\r\nSUMMARY:Bank holiday\r\nDTSTART;VALUE=DATE:20261023\r\nEND:VEVENT\r\n\
BEGIN:VEVENT\r\nUID:gone\r\nSUMMARY:Cancelled\r\nSTATUS:CANCELLED\r\nDTSTART:20261021T120000Z\r\nEND:VEVENT\r\n\
END:VCALENDAR\r\n";

    fn titles_on(day: u32) -> Vec<String> {
        let from = Utc.with_ymd_and_hms(2026, 10, day, 0, 0, 0).unwrap();
        let mut list: Vec<(DateTime<Utc>, String)> =
            occurrences(FEED, from, from + Duration::days(1))
                .into_iter()
                .filter(|o| !o.all_day)
                .map(|o| (o.start, o.title))
                .collect();
        list.sort();
        list.into_iter().map(|(_, t)| t).collect()
    }

    #[test]
    fn expands_recurrences_with_exceptions_and_moves() {
        assert_eq!(titles_on(20), ["Standup"]);
        assert_eq!(titles_on(21), ["Standup (moved)", "Dentist"]);
        assert_eq!(titles_on(22), Vec::<String>::new());
    }

    #[test]
    fn keeps_all_day_events_flagged_and_drops_cancelled_ones() {
        let from = Utc.with_ymd_and_hms(2026, 10, 23, 0, 0, 0).unwrap();
        let all = occurrences(FEED, from, from + Duration::days(1));
        let holiday = all.iter().find(|o| o.title == "Bank holiday").unwrap();
        assert!(holiday.all_day);
        assert!(!all.iter().any(|o| o.title == "Cancelled"));
    }

    #[test]
    fn finds_call_links_in_notes() {
        let from = Utc.with_ymd_and_hms(2026, 10, 21, 0, 0, 0).unwrap();
        let all = occurrences(FEED, from, from + Duration::days(1));
        let dentist = all.iter().find(|o| o.title == "Dentist").unwrap();
        assert_eq!(
            dentist.join_url.as_deref(),
            Some("https://meet.google.com/abc-defg-hij")
        );
    }

    #[test]
    fn standup_has_its_real_length_and_time_zone() {
        let from = Utc.with_ymd_and_hms(2026, 10, 20, 0, 0, 0).unwrap();
        let standup = occurrences(FEED, from, from + Duration::days(1))
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(standup.end - standup.start, Duration::minutes(15));
        assert_eq!(
            standup.start,
            Utc.with_ymd_and_hms(2026, 10, 20, 8, 30, 0).unwrap()
        );
    }
}
