//! TOML front matter between `+++` lines, followed by a markdown body.

const FENCE: &str = "+++";

/// Splits a file into (settings, body). `None` if it has no front matter.
pub fn split(text: &str) -> Option<(&str, &str)> {
    let rest = text.trim_start().strip_prefix(FENCE)?;
    let (front, body) = rest.split_once(&format!("\n{FENCE}"))?;
    Some((front.trim(), body.trim()))
}

pub fn join(front: &str, body: &str) -> String {
    format!("{FENCE}\n{}\n{FENCE}\n\n{}\n", front.trim(), body.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let text = join("name = \"x\"", "Do the thing.");
        assert_eq!(split(&text), Some(("name = \"x\"", "Do the thing.")));
    }

    #[test]
    fn needs_a_fence() {
        assert_eq!(split("just text"), None);
    }
}
