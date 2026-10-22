//! Finds the video call link in an event's text, for the Join button.

/// Video call hosts whose links make a Join button.
const CALL_HOSTS: [&str; 4] = [
    "meet.google.com",
    "zoom.us",
    "teams.microsoft.com",
    "whereby.com",
];

pub fn first_call_link(text: &str) -> Option<String> {
    text.split(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '\\'))
        .filter(|w| w.starts_with("https://"))
        .find(|w| CALL_HOSTS.iter().any(|h| w.contains(h)))
        .map(|w| w.trim_end_matches(['.', ',', ')']).to_owned())
}
