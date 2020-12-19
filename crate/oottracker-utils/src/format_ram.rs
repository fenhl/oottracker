use {
    std::{
        fmt,
        fs::File,
        io::{
            self,
            prelude::*,
        },
        path::PathBuf,
    },
    derive_more::From,
    structopt::StructOpt,
    oottracker::ram::{
        self,
        Ram,
    },
};

#[derive(StructOpt)]
struct Args {
    input: PathBuf,
}

#[derive(From)]
enum Error {
    Decode(ram::DecodeError),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Decode(e) => write!(f, "failed to read RAM: {:?}", e),
            Error::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

#[wheel::main]
fn main(args: Args) -> Result<(), Error> {
    let mut buf = Vec::with_capacity(ram::SIZE);
    File::open(args.input)?.read_to_end(&mut buf)?;
    println!("{:#?}", Ram::from_bytes(&buf)?);
    Ok(())
}
