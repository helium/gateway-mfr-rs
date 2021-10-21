use crate::{cmd::*, tests::Test};
use serde_json::json;

/// Read the slot configuration for a given slot
#[derive(Debug, StructOpt)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self) -> Result {
        let tests = [
            Test::serial(),
            Test::zone_locked(ecc608::Zone::Data),
            Test::zone_locked(ecc608::Zone::Config),
            Test::slot_config(0..=ecc608::MAX_SLOT, ecc608::SlotConfig::default(), "ecc"),
            Test::key_config(0..=ecc608::MAX_SLOT, ecc608::KeyConfig::default(), "ecc"),
            Test::MinerKey(0),
            Test::Sign(0),
            Test::Ecdh(0),
        ];
        let results: Vec<(String, Result)> = tests
            .iter()
            .map(|test| (test.to_string(), test.run()))
            .collect();

        let json_results: Vec<serde_json::Value> = results
            .iter()
            .map(|(test, result)| {
                json!({
                    "test": test,
                    "result": test_result_to_pass_fail(result),
                    "output": test_result_to_string(result),
                })
            })
            .collect();

        let json = json!({
            "result": test_results_to_pass_fail(&results),
            "tests": json_results,
        });

        print_json(&json)
    }
}

fn test_result_to_pass_fail(result: &Result) -> String {
    result.as_ref().map_or("fail", |_| "pass").to_string()
}

fn test_results_to_pass_fail(results: &[(String, Result)]) -> String {
    if results.iter().all(|(_, result)| result.is_ok()) {
        "pass"
    } else {
        "fail"
    }
    .to_string()
}

fn test_result_to_string(result: &Result) -> String {
    match result {
        Ok(()) => "ok".to_string(),
        Err(err) => format!("{:?}", err),
    }
}
