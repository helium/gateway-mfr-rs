use crate::{cmd::print_json, Device, Result};
use helium_crypto::{Keypair, Sign};
use rand::{rngs::OsRng, RngCore};
use serde_json::json;
use std::time::{Duration, Instant};

/// Run a benchmark test.
///
/// This reports number of signing operations per second a security part can
/// handle
#[derive(Debug, clap::Args)]
pub struct Cmd {
    /// Number of iterations to use for test
    #[arg(long, short, default_value_t = 100)]
    pub iterations: u32,
    /// Flag to enable durations in the results
    #[arg(long, short)]
    pub durations: bool,
}

impl Cmd {
    pub fn run(&self, device: &Device) -> Result {
        let keypair = device.get_keypair(false)?;
        let (duration, durations) = bench_sign(&keypair, self.iterations)?;
        let rate = self.iterations as f64 / duration.as_secs_f64();
        let avg_ms = duration.as_millis() as f64 / self.iterations as f64;
        let mut json = json!({
            "iterations": self.iterations,
            "avg_ms": round2(avg_ms),
            "rate": round2(rate),
        });
        if self.durations {
            json.as_object_mut().unwrap().insert("durations".to_string(), 
            json!(durations.into_iter().map(|d| d.as_millis()).collect::<Vec<_>>()));
        }
        print_json(&json)
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn bench_sign(keypair: &Keypair, iterations: u32) -> Result<(Duration, Vec<Duration>)> {
    let mut total_duration = Duration::new(0, 0);
    let mut durations = Vec::new();
    for _ in 0..iterations {
        let mut data = [0u8; 32];
        OsRng.try_fill_bytes(&mut data)?;

        let start = Instant::now();
        let _signature = keypair.sign(&data)?;
        let duration = start.elapsed();
        total_duration += duration;
        durations.push(duration);
    }
    Ok((total_duration, durations))
}
