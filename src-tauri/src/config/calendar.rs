//! The `[calendar]` section of `bridge.toml`.

use serde::Deserialize;

/// One calendar's private iCal link and the area its events belong to.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct Feed {
    pub name: String,
    pub url: String,
    pub area: String,
    /// Extra title words to skip for this calendar only.
    #[serde(default)]
    pub ignore_titles: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CalendarSettings {
    pub poll_minutes: i64,
    pub days_ahead: i64,
    /// All-day events are usually holidays and birthdays, not things to attend at a time.
    pub ignore_all_day: bool,
    /// Skip events whose title contains any of these (case-insensitive), e.g. "Focus time".
    pub ignore_titles: Vec<String>,
    pub feeds: Vec<Feed>,
}

impl Default for CalendarSettings {
    fn default() -> Self {
        Self {
            poll_minutes: 15,
            days_ahead: 14,
            ignore_all_day: true,
            ignore_titles: Vec::new(),
            feeds: Vec::new(),
        }
    }
}

impl CalendarSettings {
    pub fn skips(&self, feed: &Feed, title: &str, all_day: bool) -> bool {
        let lower = title.to_lowercase();
        let matches = |words: &[String]| {
            words
                .iter()
                .any(|w| !w.is_empty() && lower.contains(&w.to_lowercase()))
        };
        (all_day && self.ignore_all_day)
            || matches(&self.ignore_titles)
            || matches(&feed.ignore_titles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_rules_are_case_insensitive_and_per_feed() {
        let settings = CalendarSettings {
            ignore_titles: vec!["focus".to_owned()],
            ..CalendarSettings::default()
        };
        let feed = Feed {
            name: "w".to_owned(),
            url: String::new(),
            area: "work".to_owned(),
            ignore_titles: vec!["Lunch".to_owned()],
        };
        assert!(settings.skips(&feed, "Focus Time", false));
        assert!(settings.skips(&feed, "team lunch", false));
        assert!(settings.skips(&feed, "Holiday", true));
        assert!(!settings.skips(&feed, "Standup", false));
    }
}
