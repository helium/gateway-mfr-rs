pub use crate::{
    compact_key_in_slot,
    result::{bail, Error, Result},
};
pub use helium_crypto::{
    ecc608::{self, with_ecc, Ecc},
    KeyType, Keypair,
};
pub use structopt::{clap::arg_enum, StructOpt};

pub mod config;
pub mod info;
pub mod key;
pub mod provision;
pub mod test;

pub fn print_json<T: ?Sized + serde::ser::Serialize>(value: &T) -> Result {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
