use crate::{cmd::key::Cmd as KeyCmd, cmd::*};
use ecc608_linux::{KeyConfig, SlotConfig, Zone, MAX_SLOT};

/// Configures the ECC for gateway/miner use. This includes configuring slot and
/// key configs, locking the data and config zone and generating an ecc compact
/// key in slot 0.
#[derive(Debug, StructOpt)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self, ecc: &mut Ecc) -> Result {
        let slot_config = SlotConfig::default();
        let key_config = KeyConfig::default();
        for slot in 0..=MAX_SLOT {
            ecc.set_slot_config(slot, &slot_config)?;
            ecc.set_key_config(slot, &key_config)?;
        }
        ecc.set_locked(Zone::Config)?;
        ecc.set_locked(Zone::Data)?;

        let key_cmd = KeyCmd {
            slot: 0,
            generate: true,
        };
        key_cmd.run(ecc)
    }
}
