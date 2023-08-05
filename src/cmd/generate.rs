use crate::{Device, Result};

/// Generate default configuration files for the given security device.
#[derive(Debug, clap::Args)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self, device: &Device) -> Result {
        let config = device.generate_config()?;
        let toml = toml::to_string_pretty(&config)?;
        println!("{}", toml);
        Ok(())
    }
}
