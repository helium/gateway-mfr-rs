use crate::{
    device::{
        test::{self, TestResult},
        Config as DeviceConfig, GatewaySecurityDevice,
    },
    Result,
};
use helium_crypto::{tpm, KeyTag, KeyType, Keypair, Network, Sign, Verify};
use serde::Serialize;
use std::path::{Path, PathBuf};

impl GatewaySecurityDevice for gateway_security::device::tpm::Device {
    fn provision(&self) -> Result<Keypair> {
        anyhow::bail!("not supported");
    }

    fn get_config(&self) -> Result<DeviceConfig> {
        Ok(DeviceConfig::Tpm(Config {
            path: self.path.clone(),
        }))
    }

    fn get_tests(&self) -> Vec<test::Test> {
        vec![
            Test::MinerKey(self.path.clone()).into(),
            Test::Sign(self.path.clone()).into(),
            Test::Ecdh(self.path.clone()).into(),
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
    Ecdh(PathBuf),
}

impl From<Test> for test::Test {
    fn from(value: Test) -> Self {
        Self::Tpm(value)
    }
}

impl std::fmt::Display for Test {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MinerKey(key_path) => {
                f.write_fmt(format_args!("miner_key({})", key_path.display()))
            }
            Self::Sign(key_path) => f.write_fmt(format_args!("sign({})", key_path.display())),
            Self::Ecdh(key_path) => f.write_fmt(format_args!("ecdh({})", key_path.display())),
        }
    }
}

impl Test {
    pub fn run(&self) -> TestResult {
        match self {
            Self::MinerKey(key_path) => check_miner_key(key_path),
            Self::Sign(key_path) => check_sign(key_path),
            Self::Ecdh(key_path) => check_ecdh(key_path),
        }
    }
}

fn check_miner_key(key_path: &Path) -> TestResult {
    let keypair = tpm::Keypair::from_key_path(Network::MainNet, &key_path.to_string_lossy())
        .map(helium_crypto::Keypair::from)?;
    test::pass(keypair.public_key()).into()
}

fn check_sign(key_path: &Path) -> TestResult {
    const DATA: &[u8] = b"hello world";
    let keypair = tpm::Keypair::from_key_path(Network::MainNet, &key_path.to_string_lossy())
        .map(helium_crypto::Keypair::from)?;
    let signature = keypair.sign(DATA)?;
    keypair.public_key().verify(DATA, &signature)?;
    test::pass("ok").into()
}

fn check_ecdh(key_path: &Path) -> TestResult {
    use rand::rngs::OsRng;
    let keypair = tpm::Keypair::from_key_path(Network::MainNet, &key_path.to_string_lossy())
        .map(helium_crypto::Keypair::from)?;
    let other_keypair = Keypair::generate(
        KeyTag {
            network: Network::MainNet,
            key_type: KeyType::EccCompact,
        },
        &mut OsRng,
    );
    let ecc_shared_secret = keypair.ecdh(other_keypair.public_key())?;
    let other_shared_secret = other_keypair.ecdh(keypair.public_key())?;

    if ecc_shared_secret.raw_secret_bytes() != other_shared_secret.raw_secret_bytes() {
        return test::expected(
            format!("{:#02x}", ecc_shared_secret.raw_secret_bytes()),
            format!("{:#02x}", other_shared_secret.raw_secret_bytes()),
        )
        .into();
    }
    test::pass("ok").into()
}
