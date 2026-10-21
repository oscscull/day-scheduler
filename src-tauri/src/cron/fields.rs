use serde::Serialize;

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// The five positions of a standard cron expression.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Minute,
    Hour,
    Day,
    Month,
    Weekday,
}

const UNITS: [Unit; 5] = [
    Unit::Minute,
    Unit::Hour,
    Unit::Day,
    Unit::Month,
    Unit::Weekday,
];

/// One cron field with what it means, e.g. `9-18` → "09:00–18:00".
#[derive(Debug, Clone, Serialize)]
pub struct CronField {
    pub unit: Unit,
    pub value: String,
    pub means: String,
}

/// Breaks a five-field cron expression into explained parts.
/// Returns an empty list for anything that is not five fields.
pub fn fields(expr: &str) -> Vec<CronField> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != UNITS.len() {
        return Vec::new();
    }
    UNITS
        .iter()
        .zip(parts)
        .map(|(unit, value)| CronField {
            unit: *unit,
            value: value.to_owned(),
            means: meaning(*unit, value),
        })
        .collect()
}

fn meaning(unit: Unit, value: &str) -> String {
    if value == "*" {
        return "any".to_owned();
    }
    if let Some(step) = value.strip_prefix("*/") {
        return format!("every {step}");
    }
    value
        .split(',')
        .map(|part| part_meaning(unit, part))
        .collect::<Vec<_>>()
        .join(", ")
}

fn part_meaning(unit: Unit, part: &str) -> String {
    part.split_once('-').map_or_else(
        || label(unit, part),
        |(from, to)| format!("{}–{}", label(unit, from), label(unit, to)),
    )
}

fn label(unit: Unit, value: &str) -> String {
    match unit {
        Unit::Minute => format!(":{value:0>2}"),
        Unit::Hour => format!("{value:0>2}:00"),
        Unit::Day => value.to_owned(),
        Unit::Month => name(&MONTHS, value, 1),
        Unit::Weekday => name(&WEEKDAYS, value, 0),
    }
}

/// Maps a number to a name, e.g. weekday `1` → "Mon". `7` also means Sunday.
fn name(names: &[&str], value: &str, first: usize) -> String {
    value
        .parse::<usize>()
        .ok()
        .and_then(|n| n.checked_sub(first))
        .and_then(|i| names.get(i % names.len()))
        .map_or_else(|| value.to_owned(), |n| (*n).to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn means(expr: &str) -> Vec<String> {
        fields(expr).into_iter().map(|f| f.means).collect()
    }

    #[test]
    fn explains_hourly_weekdays() {
        assert_eq!(
            means("0 9-18 * * 1-5"),
            [":00", "09:00–18:00", "any", "any", "Mon–Fri"]
        );
    }

    #[test]
    fn explains_steps_lists_and_months() {
        assert_eq!(
            means("*/15 8,12 1 6 0,7"),
            ["every 15", "08:00, 12:00", "1", "Jun", "Sun, Sun"]
        );
    }

    #[test]
    fn rejects_wrong_field_count() {
        assert!(fields("on ci.failed").is_empty());
    }
}
