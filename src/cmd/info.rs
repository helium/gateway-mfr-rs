use crate::cmd::*;
use serde_json::json;

/// Get ecc chip information
#[derive(Debug, StructOpt)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self, ecc: &mut Ecc) -> Result {
        let info = ecc.get_info()?;
        let serial = ecc.get_serial()?;
        let json = json!({
            "info": format!("{:#02x}", info),
            "serial": format!("{:#02x}", serial),
        });
        print_json(&json)
    }
}
