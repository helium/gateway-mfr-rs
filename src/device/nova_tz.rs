use crate::{
    device::{
        test::{self, TestResult},
        Config as DeviceConfig, GatewaySecurityDevice,
    },
    Result,
};
use helium_crypto::{nova_tz, Keypair, Network, Sign, Verify};
use serde::Serialize;
use std::path::{Path, PathBuf};

impl GatewaySecurityDevice for gateway_security::device::nova_tz::Device {
    fn provision(&self) -> Result<Keypair> {
        anyhow::bail!("not supported")
    }

    fn get_config(&self) -> Result<DeviceConfig> {
        Ok(DeviceConfig::TrustZone(Config {
            path: self.path.clone(),
        }))
    }

    fn get_tests(&self) -> Vec<test::Test> {
        vec![
            Test::MinerKey(self.path.clone()).into(),
            Test::Sign(self.path.clone()).into(),
        ]
    }
}

#[derive(Debug, Serialize)]
pub struct Config {
    path: PathBuf,
}

#[derive(Debug)]
pub enum Test {
    MinerKey(PathBuf),
    Sign(PathBuf),
}

impl From<Test> for test::Test {
    fn from(value: Test) -> Self {
        Self::TrustZone(value)
    }
}

impl std::fmt::Display for Test {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MinerKey(key_path) => {
                f.write_fmt(format_args!("miner_key({})", key_path.to_string_lossy()))
            }
            Self::Sign(key_path) => {
                f.write_fmt(format_args!("sign({})", key_path.to_string_lossy()))
            }
        }
    }
}

impl Test {
    pub fn run(&self) -> TestResult {
        match self {
            Self::MinerKey(key_path) => check_miner_key(key_path),
            Self::Sign(key_path) => check_sign(key_path),
        }
    }
}

fn check_miner_key(key_path: &Path) -> TestResult {
    let keypair = nova_tz::Keypair::from_key_path(Network::MainNet, key_path)
        .map(helium_crypto::Keypair::from)?;
    test::pass(keypair.public_key()).into()
}

fn check_sign(key_path: &Path) -> TestResult {
    const DATA: &[u8] = b"hello world";
    let keypair = nova_tz::Keypair::from_key_path(Network::MainNet, key_path)
        .map(helium_crypto::Keypair::from)?;
    let signature = keypair.sign(DATA)?;
    keypair.public_key().verify(DATA, &signature)?;
    test::pass("ok").into()
}
