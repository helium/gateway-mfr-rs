use crate::{bail, compact_key_in_slot, Result};
use core::ops::RangeBounds;
use ecc608_linux::{Ecc, KeyConfig, SlotConfig, Zone, MAX_SLOT};
use std::fmt;

#[derive(Debug)]
pub enum Test {
    Serial,
    ZoneLocked(Zone),
    SlotConfig {
        start: u8,
        end: u8,
        config: SlotConfig,
        name: &'static str,
    },
    KeyConfig {
        start: u8,
        end: u8,
        config: KeyConfig,
        name: &'static str,
    },
    MinerKey(u8),
}

impl fmt::Display for Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serial => f.write_str("serial"),
            Self::ZoneLocked(zone) => {
                let zone_str = match zone {
                    Zone::Config => "config",
                    Zone::Data => "data",
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
        }
    }
}

impl Test {
    pub fn serial() -> Self {
        Self::Serial
    }

    pub fn zone_locked(zone: Zone) -> Self {
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
            Bound::Unbounded => MAX_SLOT,
        };

        assert!(
            begin <= end,
            "range start must not be greater than end: {:?} <= {:?}",
            begin,
            end,
        );
        assert!(
            end <= MAX_SLOT,
            "range end out of bounds: {:?} <= {:?}",
            end,
            MAX_SLOT,
        );

        (begin, end)
    }

    pub fn slot_config(
        range: impl RangeBounds<u8>,
        config: SlotConfig,
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

    pub fn key_config(range: impl RangeBounds<u8>, config: KeyConfig, name: &'static str) -> Self {
        let (start, end) = Self::get_start_end(range);
        Self::KeyConfig {
            start,
            end,
            config,
            name,
        }
    }

    pub fn run(&self, ecc: &mut Ecc) -> Result {
        match self {
            Self::Serial => check_serial(ecc),
            Self::ZoneLocked(zone) => check_zone_locked(ecc, zone),
            Self::SlotConfig {
                start, end, config, ..
            } => check_slot_configs(ecc, *start, *end, config),
            Self::KeyConfig {
                start, end, config, ..
            } => check_key_configs(ecc, *start, *end, config),
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

fn check_zone_locked(ecc: &mut Ecc, zone: &Zone) -> Result {
    match ecc.get_locked(&zone)? {
        true => Ok(()),
        _ => bail!("unlocked"),
    }
}

fn check_slot_configs(ecc: &mut Ecc, start: u8, end: u8, expected: &SlotConfig) -> Result {
    for slot in start..=end {
        match ecc.get_slot_config(slot)? {
            config if &config == expected => continue,
            _config => bail!("invalid slot: {:?}", slot),
        }
    }
    Ok(())
}

fn check_key_configs(ecc: &mut Ecc, start: u8, end: u8, expected: &KeyConfig) -> Result {
    for slot in start..=end {
        match ecc.get_key_config(slot)? {
            config if &config == expected => continue,
            _config => bail!("invalid slot: {:?}", slot),
        }
    }
    Ok(())
}

fn check_miner_key(ecc: &mut Ecc, slot: u8) -> Result {
    let _ = compact_key_in_slot(ecc, slot)?;
    Ok(())
}
