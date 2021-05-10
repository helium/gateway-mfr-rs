use crate::{bail, Result};
use ecc608_linux::{Ecc, KeyConfig, KeyType, SlotConfig, Zone};

#[derive(Debug)]
pub enum Test {
    Serial,
    ZoneLocked(Zone),
    SlotConfig(u8),
    KeyConfig(u8),
    MinerKey(u8),
}

impl Test {
    pub fn run(&self, ecc: &mut Ecc) -> Result {
        match self {
            Self::Serial => check_serial(ecc),
            Self::ZoneLocked(zone) => check_zone_locked(ecc, zone.clone()),
            Self::SlotConfig(slot) => check_slot_config(ecc, *slot),
            Self::KeyConfig(slot) => check_key_config(ecc, *slot),
            Self::MinerKey(slot) => check_miner_key(ecc, *slot),
        }
    }
}

fn check_serial(ecc: &mut Ecc) -> Result {
    let bytes = ecc.get_serial()?;
    if bytes[0] == 0x01 && bytes[1] == 0x23 && bytes[8] == 0xee {
        Ok(())
    } else {
        bail!("invalid serial: {:#02x}", bytes)
    }
}

fn check_zone_locked(ecc: &mut Ecc, zone: Zone) -> Result {
    match ecc.get_locked(zone)? {
        true => Ok(()),
        _ => bail!("unlocked"),
    }
}

fn check_slot_config(ecc: &mut Ecc, slot: u8) -> Result {
    match ecc.get_slot_config(slot)? {
        config if config == SlotConfig::default() => Ok(()),
        _config => bail!("invalid slot config"),
    }
}

fn check_key_config(ecc: &mut Ecc, slot: u8) -> Result {
    match ecc.get_key_config(slot)? {
        config if config == KeyConfig::default() => Ok(()),
        _config => bail!("invalid key config"),
    }
}

fn check_miner_key(ecc: &mut Ecc, slot: u8) -> Result {
    use helium_crypto::{ecc_compact, PUBLIC_KEY_LENGTH};
    use std::convert::TryFrom;
    let bytes = ecc.genkey(KeyType::Public, slot)?;
    let _ = ecc_compact::PublicKey::try_from(&bytes[0..PUBLIC_KEY_LENGTH])?;
    Ok(())
}
