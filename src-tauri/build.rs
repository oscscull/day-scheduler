//! Builds the Swift helper that reads the macOS Calendar app, then runs Tauri's and Based's builds.
//! Tauri ships the helper inside the app as a sidecar (`tauri.macos.conf.json`).

use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    if env::var("CARGO_CFG_TARGET_OS")? == "macos" {
        calendar_helper()?;
    }
    tauri_build::build();
    based_build::run();
    Ok(())
}

fn run(command: &mut Command) -> Result<(), Box<dyn Error>> {
    if command.status()?.success() {
        Ok(())
    } else {
        Err(format!("couldn't build the calendar helper: {command:?} failed").into())
    }
}

/// Compiles the helper for one CPU. The usage text in Info.plist is embedded so macOS can ask for
/// access even when the helper runs outside the app bundle, as in `tauri dev`.
fn compile(arch: &str, out: &Path) -> Result<(), Box<dyn Error>> {
    run(Command::new("xcrun")
        .args(["swiftc", "-O", "-target", &format!("{arch}-apple-macos14")])
        .args(["-Xlinker", "-sectcreate", "-Xlinker", "__TEXT"])
        .args(["-Xlinker", "__info_plist", "-Xlinker", "Info.plist"])
        .arg("helpers/calendars.swift")
        .arg("-o")
        .arg(out))
}

/// One universal helper, named for this build's target and for Tauri's universal bundle, which
/// looks for `-universal-apple-darwin`. Needs Xcode's command line tools.
fn calendar_helper() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=helpers/calendars.swift");
    println!("cargo:rerun-if-changed=Info.plist");
    let scratch = Path::new(&env::var("OUT_DIR")?).join("calendars");
    fs::create_dir_all(&scratch)?;
    let (arm, intel) = (scratch.join("arm64"), scratch.join("x86_64"));
    compile("arm64", &arm)?;
    compile("x86_64", &intel)?;
    let universal = scratch.join("universal");
    run(Command::new("lipo")
        .arg("-create")
        .args([&arm, &intel])
        .arg("-output")
        .arg(&universal))?;
    fs::create_dir_all("binaries")?;
    for triple in [env::var("TARGET")?, "universal-apple-darwin".to_owned()] {
        fs::copy(
            &universal,
            format!("binaries/quarterdeck-calendars-{triple}"),
        )?;
    }
    Ok(())
}
