use super::Config;
use crate::error::AppResult;

/// Built-in agent jobs, each backed by `prompts/<name>.md`.
#[derive(Debug, Clone, Copy)]
pub enum Prompt {
    Chat,
    DraftSchedule,
    ResolveUrgent,
    RunSchedule,
    CheckIn,
    FixCi,
    CiDone,
    Answer,
    Message,
    Workspace,
    NoWorkspace,
}

impl Prompt {
    const fn file(self) -> &'static str {
        match self {
            Self::Chat => "chat.md",
            Self::DraftSchedule => "draft-schedule.md",
            Self::ResolveUrgent => "resolve-urgent.md",
            Self::RunSchedule => "run-schedule.md",
            Self::CheckIn => "check-in.md",
            Self::FixCi => "fix-ci.md",
            Self::CiDone => "ci-done.md",
            Self::Answer => "answer.md",
            Self::Message => "message.md",
            Self::Workspace => "workspace.md",
            Self::NoWorkspace => "no-workspace.md",
        }
    }
}

/// Loads a prompt and fills its `{{key}}` placeholders.
pub fn render(config: &Config, prompt: Prompt, vars: &[(&str, &str)]) -> AppResult<String> {
    let template = std::fs::read_to_string(config.root().join("prompts").join(prompt.file()))?;
    Ok(fill(&template, vars))
}

fn fill(template: &str, vars: &[(&str, &str)]) -> String {
    vars.iter().fold(template.to_owned(), |text, (key, value)| {
        text.replace(&format!("{{{{{key}}}}}"), value)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_placeholders() {
        assert_eq!(
            fill("Hi {{name}}, {{name}}!", &[("name", "Oscar")]),
            "Hi Oscar, Oscar!"
        );
    }
}
