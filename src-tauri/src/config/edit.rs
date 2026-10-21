//! Changes `bridge.toml` in place, keeping the user's comments and layout. Every edit is checked
//! by loading the result; a change that would break the file is undone.

use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, value};

use super::Config;
use super::calendar::Feed;
use super::settings::{self, Area};
use crate::error::{AppError, AppResult};

fn path(config: &Config) -> std::path::PathBuf {
    config.root().join("bridge.toml")
}

/// Applies `change` to the settings file, keeping it only if the result still loads.
fn edit(config: &Config, change: impl FnOnce(&mut DocumentMut) -> AppResult<()>) -> AppResult<()> {
    let before = std::fs::read_to_string(path(config))?;
    let mut doc: DocumentMut = before
        .parse()
        .map_err(|e| AppError::BadInput(format!("bridge.toml: {e}")))?;
    change(&mut doc)?;
    std::fs::write(path(config), doc.to_string())?;
    if let Err(e) = settings::load(config) {
        std::fs::write(path(config), before)?;
        return Err(AppError::BadInput(format!(
            "that change would break bridge.toml: {e}"
        )));
    }
    Ok(())
}

fn table<'a>(doc: &'a mut DocumentMut, name: &str) -> AppResult<&'a mut Table> {
    doc.entry(name)
        .or_insert_with(|| Item::Table(Table::new()))
        .as_table_mut()
        .ok_or_else(|| AppError::BadInput(format!("[{name}] in bridge.toml isn't a table")))
}

fn tables<'a>(parent: &'a mut Table, name: &str) -> AppResult<&'a mut ArrayOfTables> {
    parent
        .entry(name)
        .or_insert_with(|| Item::ArrayOfTables(ArrayOfTables::new()))
        .as_array_of_tables_mut()
        .ok_or_else(|| AppError::BadInput(format!("{name} in bridge.toml isn't a list of tables")))
}

/// Replaces the table whose `key` equals `id`, or adds `t` at the end.
fn replace_or_push(list: &mut ArrayOfTables, key: &str, id: &str, t: Table) {
    let at = list
        .iter()
        .position(|row| row.get(key).and_then(Item::as_str) == Some(id));
    match at.and_then(|i| list.get_mut(i)) {
        Some(existing) => *existing = t,
        None => list.push(t),
    }
}

fn strings(list: &[String]) -> Item {
    value(
        list.iter()
            .map(String::as_str)
            .collect::<toml_edit::Array>(),
    )
}

/// Adds an area, or replaces the one with the same id.
pub fn upsert_area(config: &Config, area: &Area) -> AppResult<()> {
    edit(config, |doc| {
        let list = tables(doc.as_table_mut(), "areas")?;
        let mut t = Table::new();
        t.insert("id", value(&area.id));
        t.insert("name", value(&area.name));
        t.insert("color", value(&area.color));
        t.insert("node", value(&area.node));
        replace_or_push(list, "id", &area.id, t);
        Ok(())
    })
}

/// Adds a calendar feed, or replaces the one with the same name.
pub fn upsert_feed(config: &Config, feed: &Feed) -> AppResult<()> {
    edit(config, |doc| {
        let list = tables(table(doc, "calendar")?, "feeds")?;
        let mut t = Table::new();
        t.insert("name", value(&feed.name));
        t.insert("url", value(&feed.url));
        t.insert("area", value(&feed.area));
        t.insert("ignore_titles", strings(&feed.ignore_titles));
        replace_or_push(list, "name", &feed.name, t);
        Ok(())
    })
}

pub fn remove_feed(config: &Config, name: &str) -> AppResult<()> {
    edit(config, |doc| {
        tables(table(doc, "calendar")?, "feeds")?
            .retain(|f| f.get("name").and_then(Item::as_str) != Some(name));
        Ok(())
    })
}

pub fn set_calendar_ignores(
    config: &Config,
    titles: &[String],
    all_day: Option<bool>,
) -> AppResult<()> {
    edit(config, |doc| {
        let calendar = table(doc, "calendar")?;
        calendar.insert("ignore_titles", strings(titles));
        if let Some(skip) = all_day {
            calendar.insert("ignore_all_day", value(skip));
        }
        Ok(())
    })
}

/// Sends an owner's or repo's PRs to an area.
pub fn map_github(config: &Config, owner_or_repo: &str, area: &str) -> AppResult<()> {
    edit(config, |doc| {
        let github = table(doc, "github")?;
        let areas = github
            .entry("areas")
            .or_insert_with(|| Item::Table(Table::new()));
        areas[owner_or_repo] = value(area);
        Ok(())
    })
}

pub fn set_github_ignores(config: &Config, ignore: &[String]) -> AppResult<()> {
    edit(config, |doc| {
        table(doc, "github")?.insert("ignore", strings(ignore));
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_keep_comments_and_refuse_breakage() {
        let dir = std::env::temp_dir().join(format!("bridge-edit-{}", std::process::id()));
        drop(std::fs::remove_dir_all(&dir));
        let config = Config::open(dir).unwrap();
        map_github(&config, "acme", "work").unwrap();
        let feed = Feed {
            name: "home".to_owned(),
            url: "https://x/ics".to_owned(),
            area: "life".to_owned(),
            ignore_titles: vec![],
        };
        upsert_feed(&config, &feed).unwrap();
        upsert_feed(&config, &feed).unwrap();
        let text = std::fs::read_to_string(path(&config)).unwrap();
        assert!(text.contains("# Mechanical settings only"));
        let loaded = settings::load(&config).unwrap();
        assert_eq!(loaded.calendar.feeds.len(), 1);
        assert_eq!(loaded.github.area_for("acme/app"), "work");
        assert!(
            edit(&config, |doc| {
                doc["priority_limit"] = value("lots");
                Ok(())
            })
            .is_err()
        );
        assert_eq!(settings::load(&config).unwrap().priority_limit, 3);
    }
}
