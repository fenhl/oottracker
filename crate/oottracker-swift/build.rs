use std::env;

fn main() -> Result<(), cbindgen::Error> {
    cbindgen::generate_with_config(
        env::var_os("CARGO_MANIFEST_DIR").unwrap(),
        cbindgen::Config {
            language: cbindgen::Language::C,
            ..cbindgen::Config::default()
        },
    )?.write_to_file("oottracker.h");
    Ok(())
}
