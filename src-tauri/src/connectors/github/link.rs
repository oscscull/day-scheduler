//! Reads which issue or PR a GitHub link points at.

/// `owner/repo` and the number from `https://github.com/<owner>/<repo>/<section>/<n>`, where
/// `section` is `issues` or `pull`, with any trailing `/`, `#…` or `?…` ignored.
pub fn parse(link: &str, section: &str) -> Option<(String, u64)> {
    let path = link.trim().strip_prefix("https://github.com/")?;
    let mut parts = path.split('/');
    let (owner, repo) = (parts.next()?, parts.next()?);
    if parts.next()? != section || owner.is_empty() || repo.is_empty() {
        return None;
    }
    let digits: String = parts
        .next()?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    Some((format!("{owner}/{repo}"), digits.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_repo_and_number_of_its_section() {
        let pr = Some(("me/app".to_owned(), 24));
        for link in [
            "https://github.com/me/app/pull/24",
            "https://github.com/me/app/pull/24/files",
            " https://github.com/me/app/pull/24#issuecomment-1 ",
        ] {
            assert_eq!(parse(link, "pull"), pr, "{link}");
        }
        assert_eq!(parse("https://github.com/me/app/issues/24", "pull"), None);
        assert_eq!(parse("https://github.com/me/app/pull/", "pull"), None);
        assert_eq!(parse("https://example.com/me/app/pull/24", "pull"), None);
    }
}
