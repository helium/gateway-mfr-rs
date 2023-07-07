use crate::{
    device::{
        test::{self, TestResult},
        Config as DeviceConfig, GatewaySecurityDevice,
    },
    Result,
};
use helium_crypto::{KeyTag, KeyType, Keypair, Sign, Verify};
use rand::rngs::OsRng;
use serde::Serialize;
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

impl GatewaySecurityDevice for gateway_security::device::file::Device {
    fn provision(&self) -> Result<Keypair> {
        Ok(self.get_keypair(true)?)
    }

    fn get_config(&self) -> Result<DeviceConfig> {
        Ok(DeviceConfig::File(Config {
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
        Self::File(value)
    }
}

impl fmt::Display for Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MinerKey(path) => {
                f.write_fmt(format_args!("miner_key({}", path.to_string_lossy()))
            }
            Self::Sign(path) => f.write_fmt(format_args!("sign({})", path.to_string_lossy())),
            Self::Ecdh(path) => f.write_fmt(format_args!("ecdh({})", path.to_string_lossy())),
        }
    }
}

impl Test {
    pub fn run(&self) -> TestResult {
        match self {
            Self::MinerKey(path) => check_miner_key(path),
            Self::Sign(path) => check_sign(path),
            Self::Ecdh(path) => check_ecdh(path),
        }
    }
}

fn load_keypair<P: AsRef<Path>>(path: &P) -> Result<Keypair> {
    let data = fs::read(path)?;
    let keypair = Keypair::try_from(&data[..])?;
    Ok(keypair)
}

fn check_miner_key(path: &PathBuf) -> TestResult {
    let keypair = load_keypair(path)?;
    test::pass(keypair.public_key()).into()
}

fn check_sign(path: &PathBuf) -> TestResult {
    const DATA: &[u8] = b"hello world";
    let keypair = load_keypair(path)?;
    let signature = keypair.sign(DATA)?;
    keypair.public_key().verify(DATA, &signature)?;
    test::pass("ok").into()
}

fn check_ecdh(path: &PathBuf) -> TestResult {
    let keypair = load_keypair(path)?;
    let other_keypair = Keypair::generate(
        KeyTag {
            network: keypair.key_tag().network,
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
