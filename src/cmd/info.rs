use crate::cmd::*;

/// Get ecc chip information
#[derive(Debug, StructOpt)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self, ecc: &mut Ecc) -> Result {
        let info = ecc.get_info()?;
        let serial = ecc.get_serial()?;
        ptable!(
            ["Key", "Value"],
            ["Info", format!("{:#02x}", info)],
            ["Serial", format!("{:#02x}", serial)]
        );
        Ok(())
    }
}
