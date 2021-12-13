use std::{
    env,
    path::PathBuf,
};

fn main() -> Result<(), cbindgen::Error> {
    cbindgen::generate_with_config(
        env::var_os("CARGO_MANIFEST_DIR").unwrap(),
        cbindgen::Config {
            language: cbindgen::Language::C,
            ..cbindgen::Config::default()
        },
    )?.write_to_file(
        PathBuf::from(env::var_os("CARGO_TARGET_DIR").or(env::var_os("CARGO_MANIFEST_DIR")).unwrap())
            .join("target")
            .join("oottracker.h")
    );
    Ok(())
}
