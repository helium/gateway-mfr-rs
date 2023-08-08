use crate::{cmd::key::print_keypair, Device, Result};

/// Configures the security device for gateway/miner use.
#[derive(Debug, clap::Args)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self, device: &Device) -> Result {
        device.init()?;
        let keypair = device.provision()?;
        print_keypair(&keypair)
    }
}
