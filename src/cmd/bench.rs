use crate::cmd::*;
use helium_crypto::Sign;
use rand::{rngs::OsRng, RngCore};
use serde_json::json;
use std::time::{Duration, Instant};

/// Run a benchmark test.
///
/// This reports number of signing operations per second a security part can
/// handle
#[derive(Debug, StructOpt)]
pub struct Cmd {
    /// Slot to use for benchmark
    pub slot: u8,

    /// Number of iterations to use for test
    #[structopt(long, default_value = "1000")]
    pub iterations: u32,
}

impl Cmd {
    pub fn run(&self) -> Result {
        let keypair = with_ecc(|ecc| compact_key_in_slot(ecc, self.slot))?;
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
    for _ in 1..iterations {
        let mut data = [0u8; 32];
        OsRng.try_fill_bytes(&mut data)?;

        let start = Instant::now();
        let _signature = keypair.sign(&data)?;
        total_duration += start.elapsed();
    }
    Ok(total_duration)
}
