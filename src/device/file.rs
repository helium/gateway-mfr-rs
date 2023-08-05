use crate::{
    device::test::{self, TestResult},
    Result,
};
use helium_crypto::{KeyTag, KeyType, Keypair, Sign, Verify};
use http::Uri;
use rand::rngs::OsRng;
use serde::Serialize;
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Device {
    /// The file device path
    pub path: PathBuf,
}

impl Device {
    /// Parses an ecc device url of the form `ecc:<dev>[:address][?slot=<slot>]`,
    /// where <dev> is the device file name (usually begins with i2c or tty),
    /// <address> is the bus address (default 96, ignored for swi), and <slot>
    /// is the slot to use for key lookup/manipulation (default: 0)
    pub fn from_url(url: &Uri) -> Result<Self> {
        Ok(Self {
            path: url.path().into(),
        })
    }

    pub fn get_info(&self) -> Result<Info> {
        let keypair = self.get_keypair(false)?;
        let key_type = keypair.key_tag().key_type.to_string();
        let info = Info {
            r#type: key_type,
            path: self.path.clone(),
        };
        Ok(info)
    }

    pub fn get_keypair(&self, create: bool) -> Result<Keypair> {
        if !self.path.exists() || create {
            let keypair = Keypair::generate(KeyTag::default(), &mut OsRng);
            fs::write(&self.path, keypair.to_vec())?;
        }
        load_keypair(&self.path)
    }

    pub fn provision(&self) -> Result<Keypair> {
        self.get_keypair(true)
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

fn load_keypair<P: AsRef<Path>>(path: &P) -> Result<Keypair> {
    let data = fs::read(path)?;
    let keypair = Keypair::try_from(&data[..])?;
    Ok(keypair)
}

#[derive(Debug, Serialize)]
pub struct Info {
    r#type: String,
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
    Ecdh(PathBuf),
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

    if ecc_shared_secret.as_bytes() != other_shared_secret.as_bytes() {
        return test::expected(
            format!("{:#02x}", ecc_shared_secret.as_bytes()),
            format!("{:#02x}", other_shared_secret.as_bytes()),
        )
        .into();
    }
    test::pass("ok").into()
}
