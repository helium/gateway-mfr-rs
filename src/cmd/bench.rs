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
}

impl Cmd {
    pub fn run(&self, device: &Device) -> Result {
        device.init()?;
        let keypair = device.get_keypair(false)?;
        let duration = bench_sign(&keypair, self.iterations)?;
        let rate = self.iterations as f64 / duration.as_secs_f64();
        let avg_ms = duration.as_millis() as f64 / self.iterations as f64;
        let json = json!({
            "iterations": self.iterations,
            "avg_ms": round2(avg_ms),
            "rate": round2(rate),
        });
        print_json(&json)
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn bench_sign(keypair: &Keypair, iterations: u32) -> Result<Duration> {
    let mut total_duration = Duration::new(0, 0);
    for _ in 0..iterations {
        let mut data = [0u8; 32];
        OsRng.try_fill_bytes(&mut data)?;

        let start = Instant::now();
        let _signature = keypair.sign(&data)?;
        total_duration += start.elapsed();
    }
    Ok(total_duration)
}
