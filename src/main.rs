use gateway_mfr::{cmd, result::Result};
use helium_crypto::ecc608;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = env!("CARGO_BIN_NAME"), version = env!("CARGO_PKG_VERSION"), about = "Gateway Manufacturing ")]
pub struct Cli {
    /// The i2c device path
    #[structopt(long, default_value = "/dev/i2c-1")]
    path: PathBuf,

    /// The bus address
    #[structopt(long, default_value = "96")]
    address: u16,

    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, StructOpt)]
pub enum Cmd {
    Info(cmd::info::Cmd),
    Key(cmd::key::Cmd),
    Provision(cmd::provision::Cmd),
    Config(cmd::config::Cmd),
    Test(cmd::test::Cmd),
    Bench(cmd::bench::Cmd),
}

pub fn main() -> Result {
    let cli = Cli::from_args();
    cli.cmd.run(cli.path, cli.address)
}

impl Cmd {
    fn run(&self, path: PathBuf, address: u16) -> Result {
        ecc608::init(&path.to_string_lossy(), address)?;
        match self {
            Self::Info(cmd) => cmd.run(),
            Self::Key(cmd) => cmd.run(),
            Self::Provision(cmd) => cmd.run(),
            Self::Config(cmd) => cmd.run(),
            Self::Test(cmd) => cmd.run(),
            Self::Bench(cmd) => cmd.run(),
        }
    }
}
