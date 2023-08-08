use crate::{
    anyhow,
    device::{
        test::{self, TestResult},
        DeviceArgs,
    },
    Result,
};
use bytes::Bytes;
use helium_crypto::{
    ecc608::{self, key_config::KeyConfigType, with_ecc, Ecc, EccConfig},
    KeyTag, KeyType, Keypair, Network, Sign, Verify,
};
use http::Uri;
use serde::{Serialize, Serializer};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

pub use ecc608::EccConfig as FileConfig;

#[derive(Debug, Clone)]
pub struct Device {
    /// The i2c/swi device path
    pub path: PathBuf,
    /// The bus address
    pub address: u16,
    /// The ecc slot to use
    pub slot: u8,
    /// The config parameters
    pub config: Option<EccConfig>,
}

impl Device {
    /// Parses an ecc device url of the form `ecc:<dev>[:address][?slot=<slot>]`,
    /// where <dev> is the device file name (usually begins with i2c or tty),
    /// <address> is the bus address (default 96, ignored for swi), and <slot>
    /// is the slot to use for key lookup/manipulation (default: 0)
    pub fn from_url(url: &Uri) -> Result<Self> {
        let args = DeviceArgs::from_uri(url)?;
        let address = url.port_u16().unwrap_or(96);
        let slot = args.get("slot", 0)?;
        let path = url
            .host()
            .map(|dev| Path::new("/dev").join(dev))
            .ok_or_else(|| anyhow!("missing ecc device path"))?;

        let config = if let Some(config_file) = args.get_string("config") {
            let contents = fs::read_to_string(config_file)?;
            let config: EccConfig = toml::from_str(&contents)?;
            Some(config)
        } else {
            None
        };

        Ok(Self {
            path,
            address,
            slot,
            config,
        })
    }

    pub fn init(&self) -> Result {
        // Initialize the global instance if not already initialized
        Ok(ecc608::init(
            &self.path.to_string_lossy(),
            self.address,
            self.config,
        )?)
    }

    pub fn get_info(&self) -> Result<Info> {
        let info = with_ecc(|ecc: &mut Ecc| {
            ecc.get_info()
                .and_then(|info| ecc.get_serial().map(|serial| Info { info, serial }))
        })?;
        Ok(info)
    }

    pub fn get_keypair(&self, create: bool) -> Result<Keypair> {
        let keypair: Keypair = with_ecc(|ecc| {
            if create {
                generate_compact_key_in_slot(ecc, self.slot)
            } else {
                compact_key_in_slot(ecc, self.slot)
            }
        })?;
        Ok(keypair)
    }

    pub fn provision(&self) -> Result<Keypair> {
        let slot_config = ecc608::SlotConfig::default();
        let key_config = ecc608::KeyConfig::default();
        for slot in 0..=ecc608::MAX_SLOT {
            with_ecc(|ecc| ecc.set_slot_config(slot, &slot_config))?;
            with_ecc(|ecc| ecc.set_key_config(slot, &key_config))?;
        }
        with_ecc(|ecc| ecc.set_locked(ecc608::Zone::Config))?;
        with_ecc(|ecc| ecc.set_locked(ecc608::Zone::Data))?;

        self.get_keypair(true)
    }

    pub fn get_config(&self) -> Result<Config> {
        let slot_config = with_ecc(|ecc| ecc.get_slot_config(self.slot))?;
        let key_config = with_ecc(|ecc| ecc.get_key_config(self.slot))?;
        let zones = [ecc608::Zone::Config, ecc608::Zone::Data]
            .into_iter()
            .map(get_zone_config)
            .collect::<Result<Vec<ZoneConfig>>>()?;
        Ok(Config {
            slot_config,
            key_config,
            zones,
        })
    }

    pub fn generate_config(&self) -> Result<FileConfig> {
        let config = ecc608::EccConfig::from_path(&self.path.to_string_lossy())?;
        Ok(config)
    }

    pub fn get_tests(&self) -> Vec<Test> {
        vec![
            Test::zone_locked(ecc608::Zone::Data),
            Test::zone_locked(ecc608::Zone::Config),
            Test::slot_config(self.slot, ecc608::SlotConfig::default()),
            Test::key_config(self.slot, ecc608::KeyConfig::default()),
            Test::MinerKey(self.slot),
            Test::Sign(self.slot),
            Test::Ecdh(self.slot),
        ]
    }
}

fn compact_key_in_slot(ecc: &mut Ecc, slot: u8) -> Result<Keypair> {
    let keypair = ecc608::Keypair::from_ecc_slot(ecc, Network::MainNet, slot)?;
    Ok(keypair.into())
}

fn generate_compact_key_in_slot(ecc: &mut Ecc, slot: u8) -> Result<Keypair> {
    let mut try_count = 5;
    loop {
        ecc.genkey(ecc608::KeyType::Private, slot)?;

        match compact_key_in_slot(ecc, slot) {
            Ok(keypair) => return Ok(keypair),
            Err(err) if try_count == 0 => return Err(err),
            Err(_) => try_count -= 1,
        }
    }
}

fn get_zone_config(zone: ecc608::Zone) -> Result<ZoneConfig> {
    let config = with_ecc(|ecc| ecc.get_locked(&zone)).map(|locked| ZoneConfig { zone, locked })?;
    Ok(config)
}

#[derive(Debug, Serialize)]
pub struct Info {
    #[serde(serialize_with = "serialize_bytes")]
    info: Bytes,
    #[serde(serialize_with = "serialize_bytes")]
    serial: Bytes,
}

#[derive(Debug, Serialize)]
pub struct Config {
    key_config: ecc608::KeyConfig,
    slot_config: ecc608::SlotConfig,
    zones: Vec<ZoneConfig>,
}

#[derive(Debug, Serialize)]
pub struct ZoneConfig {
    #[serde(serialize_with = "serialize_zone")]
    zone: ecc608::Zone,
    locked: bool,
}

pub fn serialize_bytes<S>(bytes: &Bytes, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("{bytes:#02x}"))
}

pub fn serialize_zone<S>(zone: &ecc608::Zone, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&zone.to_string())
}

#[derive(Debug)]
pub enum Test {
    ZoneLocked(ecc608::Zone),
    SlotConfig {
        slot: u8,
        config: ecc608::SlotConfig,
    },
    KeyConfig {
        slot: u8,
        config: ecc608::KeyConfig,
    },
    MinerKey(u8),
    Sign(u8),
    Ecdh(u8),
}

impl fmt::Display for Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZoneLocked(zone) => {
                let zone_str = match zone {
                    ecc608::Zone::Config => "config",
                    ecc608::Zone::Data => "data",
                };
                f.write_fmt(format_args!("zone_locked({zone_str})"))
            }
            Self::SlotConfig { slot, .. } => f.write_fmt(format_args!("slot_config({slot})")),
            Self::KeyConfig { slot, .. } => f.write_fmt(format_args!("key_config({slot})")),
            Self::MinerKey(slot) => f.write_fmt(format_args!("miner_key({slot})")),
            Self::Sign(slot) => f.write_fmt(format_args!("sign({slot})")),
            Self::Ecdh(slot) => f.write_fmt(format_args!("ecdh({slot})")),
        }
    }
}

impl Test {
    pub fn zone_locked(zone: ecc608::Zone) -> Self {
        Self::ZoneLocked(zone)
    }

    pub fn slot_config(slot: u8, config: ecc608::SlotConfig) -> Self {
        Self::SlotConfig { slot, config }
    }

    pub fn key_config(slot: u8, config: ecc608::KeyConfig) -> Self {
        Self::KeyConfig { slot, config }
    }

    pub fn run(&self) -> TestResult {
        match self {
            Self::ZoneLocked(zone) => check_zone_locked(zone),
            Self::SlotConfig { slot, .. } => check_slot_config(*slot),
            Self::KeyConfig { slot, .. } => check_key_config(*slot),
            Self::MinerKey(slot) => check_miner_key(*slot),
            Self::Sign(slot) => check_sign(*slot),
            Self::Ecdh(slot) => check_ecdh(*slot),
        }
    }
}

fn check_zone_locked(zone: &ecc608::Zone) -> TestResult {
    match with_ecc(|ecc| ecc.get_locked(zone))? {
        true => test::pass("ok").into(),
        _ => test::expected("locked", "unlocked").into(),
    }
}

fn check<T>(name: &'static str, found: T, expected: T) -> (&'static str, test::TestOutcome)
where
    T: fmt::Display + PartialEq,
{
    let outcome = if found == expected {
        test::pass(expected)
    } else {
        test::expected(expected, found)
    };
    (name, outcome)
}

fn check_any<T>(name: &'static str, found: T) -> (&'static str, test::TestOutcome)
where
    T: fmt::Display + PartialEq,
{
    (name, test::pass(found))
}

fn check_slot_config(slot: u8) -> TestResult {
    let config = with_ecc(|ecc| ecc.get_slot_config(slot))?;
    let outcomes = [
        check("secret", config.secret(), true),
        check("encrypt_read", config.encrypt_read(), false),
        check("limited_use", config.limited_use(), false),
        check(
            "external_signatures",
            config.read_key().external_signatures(),
            true,
        ),
        check_any(
            "internal_signatures",
            config.read_key().internal_signatures(),
        ),
        check("ecdh_operation", config.read_key().ecdh_operation(), true),
    ]
    .into_iter()
    .collect::<Vec<(&'static str, test::TestOutcome)>>();
    test::checks(outcomes).into()
}

fn check_key_config_type(key_type: KeyConfigType) -> (&'static str, test::TestOutcome) {
    let outcome = if key_type == KeyConfigType::Ecc {
        test::pass("ecc")
    } else {
        test::expected("ecc", "not_ecc")
    };
    ("key_type", outcome)
}

fn check_key_config(slot: u8) -> TestResult {
    let config = with_ecc(|ecc| ecc.get_key_config(slot))?;
    let outcomes = [
        check("auth_key", config.auth_key(), 0),
        check("intrusion_disable", config.intrusion_disable(), false),
        check("x509_index", config.x509_index(), 0),
        check("private", config.private(), true),
        check("pub_info", config.pub_info(), true),
        check_any("req_random", config.req_random()),
        check("req_auth", config.req_auth(), false),
        check("lockable", config.lockable(), true),
        check_key_config_type(config.key_type()),
    ]
    .into_iter()
    .collect::<Vec<(&'static str, test::TestOutcome)>>();
    test::checks(outcomes).into()
}

fn check_miner_key(slot: u8) -> TestResult {
    let keypair = with_ecc(|ecc| compact_key_in_slot(ecc, slot))?;
    test::pass(keypair.public_key()).into()
}

fn check_sign(slot: u8) -> TestResult {
    const DATA: &[u8] = b"hello world";
    let keypair = with_ecc(|ecc| compact_key_in_slot(ecc, slot))?;
    let signature = keypair.sign(DATA)?;
    keypair.public_key().verify(DATA, &signature)?;
    test::pass("ok").into()
}

fn check_ecdh(slot: u8) -> TestResult {
    use rand::rngs::OsRng;
    let keypair = with_ecc(|ecc| compact_key_in_slot(ecc, slot))?;
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
