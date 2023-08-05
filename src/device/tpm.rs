use http::Uri;
use std::fmt;

use serde::Serialize;

use helium_crypto::{tpm, KeyTag, KeyType, Keypair, Network, Sign, Verify};

use crate::{
    device::test::{self, TestResult},
    Result,
};

#[derive(Debug, Clone)]
pub struct Device {
    /// TPM key path
    pub path: String,
}

impl Device {
    /// Parses a tpm device url of the form `tpm://tpm/<key_path>`,
    /// where <key_path> is the path to TPM KEY
    pub fn from_url(url: &Uri) -> Result<Self> {
        let path = url.path();

        Ok(Self {
            path: path.to_string(),
        })
    }

    pub fn get_info(&self) -> Result<Info> {
        Ok(Info {
            path: self.path.clone(),
        })
    }

    pub fn get_keypair(&self, create: bool) -> Result<Keypair> {
        if create {
            panic!("not supported")
        }

        let keypair = tpm::Keypair::from_key_path(Network::MainNet, self.path.as_str())
            .map(helium_crypto::Keypair::from)?;
        Ok(keypair)
    }

    pub fn provision(&self) -> Result<Keypair> {
        panic!("not supported")
    }

    pub fn get_config(&self) -> Result<Config> {
        Ok(Config {
            path: self.path.clone(),
        })
    }

    pub fn get_tests(&self) -> Vec<Test> {
        vec![
            Test::MinerKey(self.path.clone()),
            Test::Sign(self.path.clone()),
            Test::Ecdh(self.path.clone()),
        ]
    }
}

#[derive(Debug, Serialize)]
pub struct Info {
    path: String,
}

#[derive(Debug, Serialize)]
pub struct Config {
    path: String,
}

#[derive(Debug)]
pub enum Test {
    MinerKey(String),
    Sign(String),
    Ecdh(String),
}

impl fmt::Display for Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MinerKey(key_path) => f.write_fmt(format_args!("miner_key({key_path})")),
            Self::Sign(key_path) => f.write_fmt(format_args!("sign({key_path})")),
            Self::Ecdh(key_path) => f.write_fmt(format_args!("ecdh({key_path})")),
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

fn check_miner_key(key_path: &str) -> TestResult {
    let keypair = tpm::Keypair::from_key_path(Network::MainNet, key_path)
        .map(helium_crypto::Keypair::from)?;
    test::pass(keypair.public_key()).into()
}

fn check_sign(key_path: &str) -> TestResult {
    const DATA: &[u8] = b"hello world";
    let keypair = tpm::Keypair::from_key_path(Network::MainNet, key_path)
        .map(helium_crypto::Keypair::from)?;
    let signature = keypair.sign(DATA)?;
    keypair.public_key().verify(DATA, &signature)?;
    test::pass("ok").into()
}

fn check_ecdh(key_path: &str) -> TestResult {
    use rand::rngs::OsRng;
    let keypair = tpm::Keypair::from_key_path(Network::MainNet, key_path)
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

    if ecc_shared_secret.as_bytes() != other_shared_secret.as_bytes() {
        return test::expected(
            format!("{:#02x}", ecc_shared_secret.as_bytes()),
            format!("{:#02x}", other_shared_secret.as_bytes()),
        )
        .into();
    }
    test::pass("ok").into()
}
