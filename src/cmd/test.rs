use crate::{
    cmd::print_json,
    device::{
        test::{self, TestOutcome, TestResult},
        Device,
    },
    Result,
};
use serde_json::json;
use std::collections::HashMap;

/// Read the slot configuration for a given slot
#[derive(Debug, clap::Args)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self, device: &Device) -> Result {
        device.init()?;
        let tests = device.get_tests();
        let results: Vec<(String, TestResult)> = tests
            .iter()
            .map(|test| (test.to_string(), test.run()))
            .collect();
        let passed = test_results_to_pass_fail(&results);
        let json_results: Vec<(String, serde_json::Value)> = results
            .into_iter()
            .map(|(test, result)| {
                let (out_name, out_json) = test_result_to_json(&result);
                (
                    test,
                    json!({
                        "result": test_result_to_pass_fail(&result),
                        out_name: out_json,
                    }),
                )
            })
            .collect();
        let result_map: HashMap<String, serde_json::Value> = HashMap::from_iter(json_results);
        let json = json!({
            "result": passed,
            "tests": result_map,
        });

        print_json(&json)
    }
}

fn test_result_to_pass_fail(result: &TestResult) -> String {
    result
        .as_ref()
        .map(|outcome| outcome.to_string())
        .unwrap_or_else(|_| "fail".to_string())
}

fn test_results_to_pass_fail(results: &[(String, TestResult)]) -> &'static str {
    if results
        .iter()
        .all(|(_, result)| result.as_ref().map_or(false, |outcome| outcome.passed()))
    {
        "pass"
    } else {
        "fail"
    }
}

fn test_result_to_json(result: &TestResult) -> (&'static str, TestOutcome) {
    result
        .as_ref()
        .map(|outcome| ("checks", outcome.clone()))
        .unwrap_or_else(|err| ("error", test::fail(format!("{err:?}"))))
}
