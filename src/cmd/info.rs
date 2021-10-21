use crate::cmd::*;
use serde_json::json;

/// Get ecc chip information
#[derive(Debug, StructOpt)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self) -> Result {
        let (info, serial) = with_ecc(|ecc| {
            ecc.get_info()
                .and_then(|info| ecc.get_serial().map(|serial| (info, serial)))
        })?;
        let json = json!({
            "info": format!("{:#02x}", info),
            "serial": format!("{:#02x}", serial),
        });
        print_json(&json)
    }
}
