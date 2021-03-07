use {
    structopt::StructOpt,
    crate::version::version,
};

mod version;

#[derive(StructOpt)]
enum Args {
    Major,
    Minor,
    Patch,
}

#[wheel::main]
async fn main(args: Args) {
    let mut version = version().await;
    match args {
        Args::Major => version.increment_major(),
        Args::Minor => version.increment_minor(),
        Args::Patch => version.increment_patch(),
    }
    println!("new version: {}", version); //TODO edit Cargo.toml and Info.plist files
}
