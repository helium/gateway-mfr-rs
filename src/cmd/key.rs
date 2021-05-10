use crate::cmd::*;

/// Reset the ecc chip
#[derive(Debug, StructOpt)]
pub struct Cmd {
    slot: u8,
}

impl Cmd {
    pub fn run(&self, ecc: &mut Ecc) -> Result {
        let res = ecc.genkey(KeyType::Public, self.slot)?;
        println!("KEY: {:?}", res.len());
        Ok(())
    }
}
