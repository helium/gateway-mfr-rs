use gateway_mfr::{cmd, Device, Result};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = env!("CARGO_BIN_NAME"), version = env!("CARGO_PKG_VERSION"), about = "Gateway Manufacturing ")]
pub struct Cli {
    /// The security device to use
    #[structopt(long, default_value = "ecc://i2c-1")]
    device: Device,

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
        }
    }
}
