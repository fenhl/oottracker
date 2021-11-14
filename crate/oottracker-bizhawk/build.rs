use {
    std::{
        env,
        path::Path,
        process::Command,
    },
    wheel::traits::{
        IoResultExt as _,
        SyncCommandOutputExt as _,
    },
};

#[cfg(windows)]
fn main() -> wheel::Result {
    println!("cargo:rerun-if-changed=nonexistent.foo"); // check a nonexistent file to make sure build script is always run (see https://github.com/rust-lang/cargo/issues/4213 and https://github.com/rust-lang/cargo/issues/5663)
    let is_release = match &env::var("PROFILE").expect("missing PROFILE envar")[..] {
        "debug" => false,
        "release" => true,
        profile => panic!("unexpected PROFILE envar: {:?}", profile),
    };
    let source_path = if is_release {
        Path::new("../../target/release/oottracker.dll")
    } else {
        Path::new("../../target/debug/oottracker.dll")
    };
    let source_path = source_path.canonicalize().at(source_path)?;
    for target_path in &[Path::new("OotAutoTracker/src/oottracker.dll"), Path::new("OotAutoTracker/BizHawk/ExternalTools/oottracker.dll")] {
        if target_path.read_link().is_ok() { std::fs::remove_file(target_path).at(target_path)?; }
        std::os::windows::fs::symlink_file(&source_path, target_path).at(target_path)?;
    }
    let mut dotnet_command = Command::new("dotnet");
    dotnet_command.arg("build");
    if is_release {
        dotnet_command.arg("--configuration=Release");
    }
    dotnet_command.current_dir("OotAutoTracker/src");
    dotnet_command.check("dotnet")?;
    Ok(())
}

#[cfg(not(windows))]
fn main() {} // BizHawk is currently only available on Windows
