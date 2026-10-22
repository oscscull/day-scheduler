//! Runs a prepared `claude -p` command, streaming its steps into the run's log until it replies.

use std::process::Stdio;

use chrono::Utc;
use tokio::io::{AsyncBufReadExt as _, AsyncRead, AsyncReadExt as _, BufReader};
use tokio::process::Command;

use super::stream::{self, Event, Reply};
use crate::error::{AppError, AppResult};
use crate::runlog::{self, Writer};

/// The log is a courtesy for the Agent tab: a run goes ahead without one.
pub fn open_log(run: Option<i64>) -> Option<Writer> {
    let dir = runlog::dir().ok()?;
    Writer::create(&dir, run?).ok()
}

fn record(events: Vec<Event>, log: Option<&mut Writer>, reply: &mut Option<Reply>) {
    let mut log = log;
    for event in events {
        match event {
            Event::Step(step) => {
                if let Some(w) = log.as_deref_mut() {
                    // A step that couldn't be written is only missing from the display.
                    drop(w.push(step));
                }
            }
            Event::Done(done) => *reply = Some(done),
        }
    }
}

/// Reads the stream to the end. The log goes when this returns, as the run is over.
async fn read_stream(out: impl AsyncRead + Unpin, mut log: Option<Writer>) -> Option<Reply> {
    let mut lines = BufReader::new(out).lines();
    let mut reply = None;
    while let Ok(Some(line)) = lines.next_line().await {
        record(stream::parse(&line, Utc::now()), log.as_mut(), &mut reply);
    }
    reply
}

async fn read_all(err: impl AsyncRead + Unpin) -> String {
    let mut text = String::new();
    drop(BufReader::new(err).read_to_string(&mut text).await);
    text
}

/// Says what couldn't be found: the folder to run in, or the program. Both show up as `NotFound`.
fn missing(cmd: &Command, e: std::io::Error) -> AppError {
    if e.kind() != std::io::ErrorKind::NotFound {
        return e.into();
    }
    let cmd = cmd.as_std();
    if let Some(dir) = cmd.get_current_dir().filter(|d| !d.exists()) {
        return AppError::Claude(format!(
            "the folder this run belongs in is gone: {} ({e})",
            dir.display()
        ));
    }
    let program = cmd.get_program().to_string_lossy().into_owned();
    AppError::Claude(format!(
        "can't find your agent: `{program}` isn't installed where Quarterdeck looks or on your \
         login shell's PATH, or the shell was too slow to say ({e}). Set claude_command in \
         quarterdeck.toml to its full path."
    ))
}

/// Runs `cmd` and returns Claude's reply, writing its steps to `log` as they come.
pub async fn follow(mut cmd: Command, log: Option<Writer>) -> AppResult<String> {
    cmd.args(["--output-format", "stream-json", "--verbose"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = cmd.spawn().map_err(|e| missing(&cmd, e))?;
    let (out, err) = (child.stdout.take(), child.stderr.take());
    let (reply, stderr) = tokio::join!(
        async {
            match out {
                Some(o) => read_stream(o, log).await,
                None => None,
            }
        },
        async {
            match err {
                Some(e) => read_all(e).await,
                None => String::new(),
            }
        }
    );
    let status = child.wait().await?;
    match reply {
        Some(r) if !r.failed && status.success() => Ok(r.text),
        Some(r) if !r.text.is_empty() => Err(AppError::Claude(r.text)),
        _ => Err(AppError::Claude(why_it_stopped(stderr.trim(), status))),
    }
}

/// What the agent said on stderr, else how it exited.
fn why_it_stopped(stderr: &str, status: std::process::ExitStatus) -> String {
    if stderr.is_empty() {
        format!("it exited with no message ({status})")
    } else {
        stderr.to_owned()
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::time::Duration;

    use super::*;

    const SCRIPT: &str = r#"
echo '{"type":"system","subtype":"task_summary","detail":"Reading the issue"}'
sleep 1
echo '{"type":"result","subtype":"success","is_error":false,"result":"All done."}'
"#;

    #[tokio::test]
    async fn steps_land_in_the_log_while_the_run_goes() {
        let dir = std::env::temp_dir().join(format!("quarterdeck-follow-{}", std::process::id()));
        let log = Writer::create(&dir, 1).unwrap();
        let mut cmd = Command::new("sh");
        cmd.args(["-c", SCRIPT]);
        let running = tokio::spawn(follow(cmd, Some(log)));
        tokio::time::sleep(Duration::from_millis(500)).await;
        let mid = runlog::read(&dir, 1).unwrap();
        assert_eq!(mid.first().unwrap().text, "Reading the issue");
        assert_eq!(running.await.unwrap().unwrap(), "All done.");
        assert!(runlog::latest(&dir, 1).is_none());
        runlog::clear(&dir).unwrap();
    }

    #[tokio::test]
    async fn a_failed_run_reports_its_error() {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "echo 'not logged in' >&2; exit 1"]);
        let err = follow(cmd, None).await.unwrap_err();
        assert!(err.to_string().contains("not logged in"));
    }
}
