use crate::{
    bail, compact_key_in_slot,
    ecc608::{self, with_ecc},
    Keypair, Result,
};
use core::ops::RangeBounds;
use helium_crypto::{KeyTag, KeyType, Network, Sign, Verify};
use std::fmt;

#[derive(Debug)]
pub enum Test {
    Serial,
    ZoneLocked(ecc608::Zone),
    SlotConfig {
        start: u8,
        end: u8,
        config: ecc608::SlotConfig,
        name: &'static str,
    },
    KeyConfig {
        start: u8,
        end: u8,
        config: ecc608::KeyConfig,
        name: &'static str,
    },
    MinerKey(u8),
    Sign(u8),
    Ecdh(u8),
}

impl fmt::Display for Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serial => f.write_str("serial"),
            Self::ZoneLocked(zone) => {
                let zone_str = match zone {
                    ecc608::Zone::Config => "config",
                    ecc608::Zone::Data => "data",
                };
                f.write_fmt(format_args!("zone_locked({})", zone_str))
            }
            Self::SlotConfig {
                start, end, name, ..
            } => f.write_fmt(format_args!("slot_config({}..={}, {})", start, end, name)),
            Self::KeyConfig {
                start, end, name, ..
            } => f.write_fmt(format_args!("key_config({}..={}, {})", start, end, name)),
            Self::MinerKey(slot) => f.write_fmt(format_args!("miner_key({})", slot)),
            Self::Sign(slot) => f.write_fmt(format_args!("sign({})", slot)),
            Self::Ecdh(slot) => f.write_fmt(format_args!("ecdh({})", slot)),
        }
    }
}

impl Test {
    pub fn serial() -> Self {
        Self::Serial
    }

    pub fn zone_locked(zone: ecc608::Zone) -> Self {
        Self::ZoneLocked(zone)
    }

    fn get_start_end(range: impl RangeBounds<u8>) -> (u8, u8) {
        use core::ops::Bound;

        let begin = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n.checked_sub(1).expect("out of bound"),
            Bound::Unbounded => ecc608::MAX_SLOT,
        };

        assert!(
            begin <= end,
            "range start must not be greater than end: {:?} <= {:?}",
            begin,
            end,
        );
        assert!(
            end <= ecc608::MAX_SLOT,
            "range end out of bounds: {:?} <= {:?}",
            end,
            ecc608::MAX_SLOT,
        );

        (begin, end)
    }

    pub fn slot_config(
        range: impl RangeBounds<u8>,
        config: ecc608::SlotConfig,
        name: &'static str,
    ) -> Self {
        let (start, end) = Self::get_start_end(range);
        Self::SlotConfig {
            start,
            end,
            config,
            name,
        }
    }

    pub fn key_config(
        range: impl RangeBounds<u8>,
        config: ecc608::KeyConfig,
        name: &'static str,
    ) -> Self {
        let (start, end) = Self::get_start_end(range);
        Self::KeyConfig {
            start,
            end,
            config,
            name,
        }
    }

    pub fn run(&self) -> Result {
        match self {
            Self::Serial => check_serial(),
            Self::ZoneLocked(zone) => check_zone_locked(zone),
            Self::SlotConfig {
                start, end, config, ..
            } => check_slot_configs(*start, *end, config),
            Self::KeyConfig {
                start, end, config, ..
            } => check_key_configs(*start, *end, config),
            Self::MinerKey(slot) => check_miner_key(*slot),
            Self::Sign(slot) => check_sign(*slot),
            Self::Ecdh(slot) => check_ecdh(*slot),
        }
    }
}

fn check_serial() -> Result {
    let bytes = with_ecc(|ecc| ecc.get_serial())?;
    if bytes[0] == 0x01 && bytes[1] == 0x23 && bytes[8] == 0xee {
        Ok(())
    } else {
        bail!("invalid serial: {:#02x}", bytes)
    }
}

fn check_zone_locked(zone: &ecc608::Zone) -> Result {
    match with_ecc(|ecc| ecc.get_locked(zone))? {
        true => Ok(()),
        _ => bail!("unlocked"),
    }
}

fn check_slot_configs(start: u8, end: u8, expected: &ecc608::SlotConfig) -> Result {
    for slot in start..=end {
        match with_ecc(|ecc| ecc.get_slot_config(slot))? {
            config if &config == expected => continue,
            _config => bail!("invalid slot: {:?}", slot),
        }
    }
    Ok(())
}

fn check_key_configs(start: u8, end: u8, expected: &ecc608::KeyConfig) -> Result {
    for slot in start..=end {
        match with_ecc(|ecc| ecc.get_key_config(slot))? {
            config if &config == expected => continue,
            _config => bail!("invalid slot: {:?}", slot),
        }
    }
    Ok(())
}

fn check_miner_key(slot: u8) -> Result {
    let _ = with_ecc(|ecc| compact_key_in_slot(ecc, slot))?;
    Ok(())
}

fn check_sign(slot: u8) -> Result {
    const DATA: &[u8] = b"hello world";
    let keypair = with_ecc(|ecc| compact_key_in_slot(ecc, slot))?;
    let signature = keypair.sign(DATA)?;
    keypair.public_key().verify(DATA, &signature)?;
    Ok(())
}

fn check_ecdh(slot: u8) -> Result {
    use rand::rngs::OsRng;
    let keypair = with_ecc(|ecc| compact_key_in_slot(ecc, slot))?;
    let other_keypair = Keypair::generate(
        KeyTag {
            network: Network::MainNet,
            key_type: KeyType::EccCompact,
        },
        &mut OsRng,
    );
    let ecc_shared_secret = keypair.ecdh(other_keypair.public_key())?;
    let other_shared_secret = other_keypair.ecdh(keypair.public_key())?;

    if ecc_shared_secret.as_bytes() != other_shared_secret.as_bytes() {
        bail!("invalid ecdh shared secret");
    }
    Ok(())
}
