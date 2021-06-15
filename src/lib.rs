pub mod cmd;
pub mod result;
pub mod tests;

pub use result::{anyhow, bail, Error, Result};

use ecc608_linux::{Ecc, KeyType};
use helium_crypto::PublicKey;

pub(crate) fn compact_key_in_slot(ecc: &mut Ecc, slot: u8) -> Result<PublicKey> {
    use helium_crypto::ecc_compact;
    use std::convert::TryFrom;
    // Start with the "decompressed" sec1 tag since the ecc does not include it.
    let mut key_bytes = vec![4u8];
    // Add the keybytes from the slot.
    key_bytes.extend_from_slice(ecc.genkey(KeyType::Public, slot)?.as_ref());
    let public_key = PublicKey::from(ecc_compact::PublicKey::try_from(key_bytes.as_ref())?);
    Ok(public_key)
}
