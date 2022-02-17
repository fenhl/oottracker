#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        fs::File,
        io::{
            self,
            prelude::*,
        },
        path::PathBuf,
    },
    thiserror::Error,
    oottracker::ram::{
        self,
        Ram,
    },
};

#[derive(clap::Parser)]
#[clap(version)]
struct Args {
    #[clap(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)] Decode(#[from] ram::DecodeError),
    #[error(transparent)] Io(#[from] io::Error),
}

#[wheel::main]
fn main(args: Args) -> Result<(), Error> {
    let mut buf = Vec::with_capacity(ram::SIZE);
    File::open(args.input)?.read_to_end(&mut buf)?;
    println!("{:#?}", Ram::from_bytes(&buf)?);
    Ok(())
}
