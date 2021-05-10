#[macro_use]
extern crate prettytable;

pub mod cmd;
pub mod result;
pub mod tests;

pub use result::{anyhow, bail, Error, Result};
