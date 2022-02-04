use crate::{cmd::print_json, Device, Result};
use structopt::StructOpt;

/// Gets the zone, slot or key config for a given ecc slot
#[derive(Debug, StructOpt)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self, device: &Device) -> Result {
        let config = device.get_config()?;
        print_json(&config)
    }
}
