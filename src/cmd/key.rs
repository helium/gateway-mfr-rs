use crate::{cmd::print_json, Device, Result};
use angry_purple_tiger::AnimalName;
use helium_crypto::Keypair;
use serde_json::json;
use structopt::StructOpt;

/// Prints public key information for a given slot.
///
/// WARNING: Using the generate option will generate a new keypair in the given
/// slot.
#[derive(Debug, StructOpt)]
pub struct Cmd {
    /// Generate a new private key in the slot. WARNING: This will overwrite the
    /// existing key in the slot
    #[structopt(long)]
    pub generate: bool,
}

impl Cmd {
    pub fn run(&self, device: &Device) -> Result {
        let keypair = device.get_keypair(self.generate)?;
        print_keypair(&keypair)
    }
}

pub(crate) fn print_keypair(keypair: &Keypair) -> Result {
    let public_key_str = keypair.public_key().to_string();
    let json = json!({
        "key": public_key_str,
        "name": public_key_str.parse::<AnimalName>()?.to_string(),
    });
    print_json(&json)
}
