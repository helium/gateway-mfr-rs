pub mod cmd;
pub mod result;
pub mod tests;

pub use result::{anyhow, bail, Error, Result};

use helium_crypto::{ecc608, Keypair, Network};

pub fn compact_key_in_slot(ecc: &mut ecc608::Ecc, slot: u8) -> Result<Keypair> {
    let keypair = ecc608::Keypair::from_ecc_slot(ecc, Network::MainNet, slot)?;
    Ok(keypair.into())
}
