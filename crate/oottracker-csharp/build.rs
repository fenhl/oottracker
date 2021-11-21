use std::{
    env,
    fs,
    path::Path,
};

fn main() {
    let [major, minor, patch, _] = winver::get_file_version_info("../oottracker-bizhawk/OotAutoTracker/BizHawk/EmuHawk.exe").unwrap();
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join("bizhawk-version.txt"), format!("{}.{}.{}", major, minor, patch)).unwrap();
}
