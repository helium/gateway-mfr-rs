use clap::Parser;
use gateway_mfr::{cmd, Device, Result};

#[derive(Debug, Parser)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(name = env!("CARGO_BIN_NAME"))]
pub struct Cli {
    /// The security device to use.
    ///
    /// The URL for the security device is dependent on the device type being
    /// used.
    ///
    /// Examples:
    ///
    /// ecc608 - "ecc://i2c-1", "ecc://i2c-1:96?slot=0"
    /// file - "file:///etc/keypair.bin"\n
    /// tpm - "tpm://tpm/<key_path>"
    #[arg(long, verbatim_doc_comment)]
    device: Device,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, clap::Subcommand)]
pub enum Cmd {
    Info(cmd::info::Cmd),
    Key(cmd::key::Cmd),
    Provision(cmd::provision::Cmd),
    Config(cmd::config::Cmd),
    Test(cmd::test::Cmd),
    Bench(cmd::bench::Cmd),
    Generate(cmd::generate::Cmd),
}

pub fn main() -> Result {
    let cli = Cli::parse();
    cli.cmd.run(&cli.device)
}

impl Cmd {
    fn run(&self, device: &Device) -> Result {
        match self {
            Self::Info(cmd) => cmd.run(device),
            Self::Key(cmd) => cmd.run(device),
            Self::Provision(cmd) => cmd.run(device),
            Self::Config(cmd) => cmd.run(device),
            Self::Test(cmd) => cmd.run(device),
            Self::Bench(cmd) => cmd.run(device),
            Self::Generate(cmd) => cmd.run(device),
        }
    }
}
