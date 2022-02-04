use crate::{cmd::print_json, device::Device, Result};
use structopt::StructOpt;

/// Get ecc chip information
#[derive(Debug, StructOpt)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self, device: &Device) -> Result {
        let info = device.get_info()?;
        print_json(&info)
    }
}
