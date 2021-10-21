use crate::{cmd::key::Cmd as KeyCmd, cmd::*};

/// Configures the ECC for gateway/miner use. This includes configuring slot and
/// key configs, locking the data and config zone and generating an ecc compact
/// key in slot 0.
#[derive(Debug, StructOpt)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self) -> Result {
        let slot_config = ecc608::SlotConfig::default();
        let key_config = ecc608::KeyConfig::default();
        for slot in 0..=ecc608::MAX_SLOT {
            with_ecc(|ecc| ecc.set_slot_config(slot, &slot_config))?;
            with_ecc(|ecc| ecc.set_key_config(slot, &key_config))?;
        }
        with_ecc(|ecc| ecc.set_locked(ecc608::Zone::Config))?;
        with_ecc(|ecc| ecc.set_locked(ecc608::Zone::Data))?;

        let key_cmd = KeyCmd {
            slot: 0,
            generate: true,
        };
        key_cmd.run()
    }
}
