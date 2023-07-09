use crate::{cmd::print_json, device::GatewaySecurityDevice, Device, Result};

/// Gets the security device configuration
#[derive(Debug, clap::Args)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self, device: &Device) -> Result {
        let config = device.get_config()?;
        print_json(&config)
    }
}
