use http::Uri;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Serialize;

use helium_crypto::{nova_tz, Keypair, Network, Sign, Verify};

use crate::{
    device::test::{self, TestResult},
    Result,
};

#[derive(Debug, Clone)]
pub struct Device {
    /// TrustZone keyblob path
    pub path: PathBuf,
}

impl Device {
    /// Parses a trustzone device url of the form `nova-tz://rsa/<key_path>`,
    /// where <key_path> is the path to TrustZone keyblob
    pub fn from_url(url: &Uri) -> Result<Self> {
        let path = url.path();

        Ok(Self { path: path.into() })
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

        let keypair = nova_tz::Keypair::from_key_path(Network::MainNet, &self.path)
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
        ]
    }
}

#[derive(Debug, Serialize)]
pub struct Info {
    path: PathBuf,
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

impl fmt::Display for Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
