use crate::{cmd::key::print_keypair, Device, Result};
use structopt::StructOpt;

/// Configures the ECC for gateway/miner use. This includes configuring slot and
/// key configs for †he given slot, locking the data and config zone and generating an ecc compact
/// key in the configured slot.
#[derive(Debug, StructOpt)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self, device: &Device) -> Result {
        let keypair = device.provision()?;
        print_keypair(&keypair)
    }
}
