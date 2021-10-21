use crate::cmd::*;
use angry_purple_tiger::AnimalName;
use serde_json::json;

/// Prints public key information for a given slot.
///
/// WARNING: Using the generate option will generate a new keypair in the given
/// slot.
#[derive(Debug, StructOpt)]
pub struct Cmd {
    /// Slot to generate public key for
    pub slot: u8,
    /// Generate a new private key in the slot. WARNING: This will overwrite the
    /// existing key in the slot
    #[structopt(long)]
    pub generate: bool,
}

impl Cmd {
    pub fn run(&self) -> Result {
        let keypair: Keypair = with_ecc(|ecc| {
            if self.generate {
                generate_compact_key_in_slot(ecc, self.slot)
            } else {
                compact_key_in_slot(ecc, self.slot)
            }
        })?;

        let public_key_str = keypair.public_key().to_string();

        let json = json!({
            "slot": self.slot,
            "key": public_key_str,
            "name": public_key_str.parse::<AnimalName>()?.to_string(),
        });
        print_json(&json)
    }
}

fn generate_compact_key_in_slot(ecc: &mut Ecc, slot: u8) -> Result<Keypair> {
    let mut try_count = 5;
    loop {
        ecc.genkey(ecc608::KeyType::Private, slot)?;

        match compact_key_in_slot(ecc, slot) {
            Ok(keypair) => return Ok(keypair),
            Err(err) if try_count == 0 => return Err(err),
            Err(_) => try_count -= 1,
        }
    }
}
