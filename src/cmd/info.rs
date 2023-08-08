use crate::{cmd::print_json, device::Device, Result};

/// Get ecc chip information
#[derive(Debug, clap::Args)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self, device: &Device) -> Result {
        device.init()?;
        let info = device.get_info()?;
        print_json(&info)
    }
}
