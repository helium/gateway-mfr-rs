pub mod bench;
pub mod config;
pub mod generate;
pub mod info;
pub mod key;
pub mod provision;
pub mod test;

pub fn print_json<T: ?Sized + serde::ser::Serialize>(value: &T) -> crate::Result {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
