use crate::{cmd::*, tests::Test};
use ecc608_linux::Zone;
use prettytable::Table;

/// Read the slot configuration for a given slot
#[derive(Debug, StructOpt)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self, ecc: &mut Ecc) -> Result {
        let tests = [
            Test::Serial,
            Test::ZoneLocked(Zone::Data),
            Test::ZoneLocked(Zone::Config),
            Test::SlotConfig(0),
            Test::KeyConfig(0),
            Test::MinerKey(0),
        ];
        let results: Vec<(&Test, Result)> =
            tests.iter().map(|test| (test, test.run(ecc))).collect();

        let mut table = Table::new();
        table.add_row(row!["Test", "Result"]);
        for (test, test_result) in &results {
            table.add_row(row![
                format!("{:?}", test),
                test_result_to_string(test_result)
            ]);
        }
        table.printstd();
        Ok(())
    }
}

fn test_result_to_string(result: &Result) -> String {
    match result {
        Ok(()) => "ok".to_string(),
        Err(err) => err.to_string(),
    }
}
