use crate::cmd::*;
use ecc608_linux::Zone;
use serde_json::json;

/// Gets the zone, slot or key config for a given ecc slot
#[derive(Debug, StructOpt)]
pub enum Cmd {
    Key(ConfigKey),
    Slot(ConfigSlot),
}

/// Gets the key configuration for a given slot
#[derive(Debug, StructOpt)]
pub struct ConfigKey {
    slot: u8,
}

/// Gets the slot configuration for a given slot
#[derive(Debug, StructOpt)]
pub struct ConfigSlot {
    slot: u8,
}

/// Gets the configuration for a given zone (data or config)
#[derive(Debug, StructOpt)]
pub struct ConfigZone {
    zone: Zone,
}

impl Cmd {
    pub fn run(&self, ecc: &mut Ecc) -> Result {
        match self {
            Self::Key(cmd) => cmd.run(ecc),
            Self::Slot(cmd) => cmd.run(ecc),
        }
    }
}

impl ConfigKey {
    pub fn run(&self, ecc: &mut Ecc) -> Result {
        let config = ecc.get_key_config(self.slot)?;
        let json = json!({
            "slot": self.slot,
            "key_config": config,
        });
        print_json(&json)
    }
}

impl ConfigSlot {
    pub fn run(&self, ecc: &mut Ecc) -> Result {
        let config = ecc.get_slot_config(self.slot)?;
        let json = json!({
            "slot": self.slot,
            "slot_config": config,
        });
        print_json(&json)
    }
}

impl ConfigZone {
    pub fn run(&self, ecc: &mut Ecc) -> Result {
        let locked = ecc.get_locked(&self.zone)?;
        let json = json!({
            "zone": self.zone.to_string(),
            "locked": locked,
        });
        print_json(&json)
    }
}
