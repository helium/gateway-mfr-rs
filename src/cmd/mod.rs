pub use crate::result::{bail, Error, Result};
pub use ecc608_linux::{Ecc, KeyType};
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
