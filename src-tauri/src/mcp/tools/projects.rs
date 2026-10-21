use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{NoArgs, Tool, tool};
use crate::config::edit;
use crate::config::projects::Project;
use crate::error::{AppError, AppResult};

const SET: &str = "Tells Quarterdeck where a GitHub repo is checked out on this machine, e.g. repo \
\"me/webapp\", path \"~/code/webapp\". Work Quarterdeck starts on that repo's PRs (fixing CI, acting \
on the user's answers) then runs in that project, in a git worktree of its own per item, with the \
project's own Claude settings and permissions. Changes the path if the repo is already set.";

pub fn tools() -> Vec<Tool> {
    vec![
        tool(
            "list_projects",
            "Repos Quarterdeck knows a checkout of, and where.",
            list,
        ),
        tool("set_project", SET, set),
        tool(
            "remove_project",
            "Forgets a repo's checkout, by `owner/repo`.",
            remove,
        ),
    ]
}

fn list(ctx: &Ctx, _: NoArgs) -> AppResult<Vec<Project>> {
    Ok(ctx.settings()?.projects)
}

fn set(ctx: &Ctx, project: Project) -> AppResult<()> {
    if !project.dir().join(".git").exists() {
        return Err(AppError::BadInput(format!(
            "{} isn't a git checkout",
            project.dir().display()
        )));
    }
    edit::upsert_project(&ctx.config, &project)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RepoArgs {
    repo: String,
}

fn remove(ctx: &Ctx, args: RepoArgs) -> AppResult<()> {
    edit::remove_project(&ctx.config, &args.repo)
}
