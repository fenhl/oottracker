use std::{
    env,
    io,
    path::Path,
    process::Command,
};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=nonexistent.foo"); // check a nonexistent file to make sure build script is always run (see https://github.com/rust-lang/cargo/issues/4213 and https://github.com/rust-lang/cargo/issues/5663)
    let source_path = match &env::var("PROFILE").expect("missing PROFILE envar")[..] {
        "debug" => Path::new("../../target/debug/oottracker.dll"),
        "release" => Path::new("../../target/release/oottracker.dll"),
        profile => panic!("unexpected PROFILE envar: {:?}", profile),
    }.canonicalize()?;
    for target_path in &[Path::new("OotAutoTracker/src/oottracker.dll"), Path::new("OotAutoTracker/BizHawk/ExternalTools/oottracker.dll")] {
        if target_path.read_link().is_ok() { std::fs::remove_file(target_path)?; }
        std::os::windows::fs::symlink_file(&source_path, target_path)?;
    }
    assert!(Command::new("dotnet").arg("build").arg("OotAutoTracker.csproj").current_dir("OotAutoTracker/src").status()?.success());
    Ok(())
}
