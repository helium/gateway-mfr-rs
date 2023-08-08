use crate::{anyhow, Result};
use helium_crypto::Keypair;
use http::Uri;
use serde::Serialize;
use std::{collections::HashMap, str::FromStr};

#[cfg(feature = "ecc608")]
mod ecc;
mod file;
#[cfg(feature = "nova-tz")]
mod nova_tz;
#[cfg(feature = "tpm")]
mod tpm;

/// A security device to work with. Security devices come in all forms. This
/// abstracts them into one with a well defined interface for doing what this
/// tool needs to do with them.
#[derive(Debug, Clone)]
pub enum Device {
    #[cfg(feature = "ecc608")]
    Ecc(ecc::Device),
    #[cfg(feature = "tpm")]
    Tpm(tpm::Device),
    #[cfg(feature = "nova-tz")]
    TrustZone(nova_tz::Device),
    File(file::Device),
}

pub struct DeviceArgs(HashMap<String, String>);

/// Represents the configuration state for the given security device. This
/// information should include enough detail to convey that the security device
/// is "locked" so key material can be written to it.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Config {
    #[cfg(feature = "ecc608")]
    Ecc(ecc::Config),
    #[cfg(feature = "tpm")]
    Tpm(tpm::Config),
    #[cfg(feature = "nova-tz")]
    TrustZone(nova_tz::Config),
    File(file::Config),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum FileConfig {
    #[cfg(feature = "ecc608")]
    Ecc(ecc::FileConfig),
}

pub mod test {
    use crate::Result;

    #[cfg(feature = "ecc608")]
    use crate::device::ecc;
    use crate::device::file;
    #[cfg(feature = "nova-tz")]
    use crate::device::nova_tz;
    #[cfg(feature = "tpm")]
    use crate::device::tpm;

    use serde::Serialize;
    use std::{collections::HashMap, fmt};

    /// Represents a single test for a given device
    pub enum Test {
        #[cfg(feature = "ecc608")]
        Ecc(ecc::Test),
        #[cfg(feature = "tpm")]
        Tpm(tpm::Test),
        #[cfg(feature = "nova-tz")]
        TrustZone(nova_tz::Test),
        File(file::Test),
    }

    #[derive(Debug, Serialize, Clone)]
    #[serde(untagged)]
    pub enum TestOutcome {
        Pass(String),
        Fail(String),
        Expect { found: String, expected: String },
        Checks(HashMap<&'static str, TestOutcome>),
    }

    pub type TestResult = Result<TestOutcome>;

    impl From<TestOutcome> for TestResult {
        fn from(v: TestOutcome) -> Self {
            Ok(v)
        }
    }

    pub fn pass<T: ToString>(msg: T) -> TestOutcome {
        TestOutcome::Pass(msg.to_string())
    }

    pub fn fail<T: ToString>(msg: T) -> TestOutcome {
        TestOutcome::Fail(msg.to_string())
    }

    pub fn expected<T: ToString>(expected: T, found: T) -> TestOutcome {
        TestOutcome::Expect {
            expected: expected.to_string(),
            found: found.to_string(),
        }
    }

    pub fn checks<T: IntoIterator<Item = (&'static str, TestOutcome)>>(checks: T) -> TestOutcome {
        TestOutcome::Checks(HashMap::from_iter(checks))
    }

    impl Test {
        pub fn run(&self) -> TestResult {
            match self {
                #[cfg(feature = "ecc608")]
                Self::Ecc(test) => test.run(),
                #[cfg(feature = "tpm")]
                Self::Tpm(test) => test.run(),
                #[cfg(feature = "nova-tz")]
                Self::TrustZone(test) => test.run(),
                Self::File(test) => test.run(),
            }
        }
    }

    impl fmt::Display for Test {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                #[cfg(feature = "ecc608")]
                Self::Ecc(test) => test.fmt(f),
                #[cfg(feature = "tpm")]
                Self::Tpm(test) => test.fmt(f),
                #[cfg(feature = "nova-tz")]
                Self::TrustZone(test) => test.fmt(f),
                Self::File(test) => test.fmt(f),
            }
        }
    }

    impl TestOutcome {
        pub fn passed(&self) -> bool {
            match self {
                Self::Pass(_) => true,
                Self::Expect { .. } => false,
                Self::Fail(_) => false,
                Self::Checks(tests) => tests.iter().all(|(_, outcome)| outcome.passed()),
            }
        }
    }

    impl fmt::Display for TestOutcome {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.passed() {
                f.write_str("pass")
            } else {
                f.write_str("fail")
            }
        }
    }
}

impl FromStr for Device {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self> {
        let url: Uri = s
            .parse()
            .map_err(|err| anyhow!("invalid device url \"{s}\": {err:?}"))?;
        match url.scheme_str() {
            #[cfg(feature = "ecc608")]
            Some("ecc") => Ok(Self::Ecc(ecc::Device::from_url(&url)?)),
            #[cfg(feature = "tpm")]
            Some("tpm") => Ok(Self::Tpm(tpm::Device::from_url(&url)?)),
            #[cfg(feature = "nova-tz")]
            Some("nova-tz") => Ok(Self::TrustZone(nova_tz::Device::from_url(&url)?)),
            Some("file") | None => Ok(Self::File(file::Device::from_url(&url)?)),
            _ => Err(anyhow!("invalid device url \"{s}\"")),
        }
    }
}

impl DeviceArgs {
    #[cfg(feature = "ecc608")]
    pub(crate) fn from_uri(url: &Uri) -> Result<Self> {
        let args = url
            .query()
            .map_or_else(
                || Ok(HashMap::new()),
                serde_urlencoded::from_str::<HashMap<String, String>>,
            )
            .map_err(|err| anyhow!("invalid device url \"{url}\": {err:?}"))?;
        Ok(Self(args))
    }

    pub fn get_string(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }

    pub fn get<T>(&self, name: &str, default: T) -> Result<T>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        self.0
            .get(name)
            .map(|s| s.parse::<T>())
            .unwrap_or(Ok(default))
            .map_err(|err| anyhow!("invalid uri argument for {name}: {err:?}"))
    }
}

impl Device {
    pub fn init(&self) -> Result {
        match self {
            #[cfg(feature = "ecc608")]
            Self::Ecc(device) => device.init(),
            _ => Ok(()),
        }
    }

    pub fn get_info(&self) -> Result<Info> {
        let info = match self {
            #[cfg(feature = "ecc608")]
            Self::Ecc(device) => Info::Ecc(device.get_info()?),
            #[cfg(feature = "tpm")]
            Self::Tpm(device) => Info::Tpm(device.get_info()?),
            #[cfg(feature = "nova-tz")]
            Self::TrustZone(device) => Info::TrustZone(device.get_info()?),
            Self::File(device) => Info::File(device.get_info()?),
        };
        Ok(info)
    }

    pub fn get_config(&self) -> Result<Config> {
        let config = match self {
            #[cfg(feature = "ecc608")]
            Self::Ecc(device) => Config::Ecc(device.get_config()?),
            #[cfg(feature = "tpm")]
            Self::Tpm(device) => Config::Tpm(device.get_config()?),
            #[cfg(feature = "nova-tz")]
            Self::TrustZone(device) => Config::TrustZone(device.get_config()?),
            Self::File(device) => Config::File(device.get_config()?),
        };
        Ok(config)
    }

    pub fn get_keypair(&self, create: bool) -> Result<Keypair> {
        let keypair = match self {
            #[cfg(feature = "ecc608")]
            Self::Ecc(device) => device.get_keypair(create)?,
            #[cfg(feature = "tpm")]
            Self::Tpm(device) => device.get_keypair(create)?,
            #[cfg(feature = "nova-tz")]
            Self::TrustZone(device) => device.get_keypair(create)?,
            Self::File(device) => device.get_keypair(create)?,
        };
        Ok(keypair)
    }

    pub fn provision(&self) -> Result<Keypair> {
        let keypair = match self {
            #[cfg(feature = "ecc608")]
            Self::Ecc(device) => device.provision()?,
            #[cfg(feature = "tpm")]
            Self::Tpm(device) => device.provision()?,
            #[cfg(feature = "nova-tz")]
            Self::TrustZone(device) => device.provision()?,
            Self::File(device) => device.provision()?,
        };
        Ok(keypair)
    }

    pub fn get_tests(&self) -> Vec<test::Test> {
        match self {
            #[cfg(feature = "ecc608")]
            Self::Ecc(device) => device
                .get_tests()
                .into_iter()
                .map(test::Test::Ecc)
                .collect(),
            #[cfg(feature = "tpm")]
            Self::Tpm(device) => device
                .get_tests()
                .into_iter()
                .map(test::Test::Tpm)
                .collect(),
            #[cfg(feature = "nova-tz")]
            Self::TrustZone(device) => device
                .get_tests()
                .into_iter()
                .map(test::Test::TrustZone)
                .collect(),
            Self::File(device) => device
                .get_tests()
                .into_iter()
                .map(test::Test::File)
                .collect(),
        }
    }

    pub fn generate_config(&self) -> Result<FileConfig> {
        let config = match self {
            #[cfg(feature = "ecc608")]
            Self::Ecc(device) => FileConfig::Ecc(device.generate_config()?),
            _ => return Err(anyhow!("device does not support config generation")),
        };
        Ok(config)
    }
}

/// Represents useful device information like model and serial number.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Info {
    #[cfg(feature = "ecc608")]
    Ecc(ecc::Info),
    #[cfg(feature = "tpm")]
    Tpm(tpm::Info),
    #[cfg(feature = "nova-tz")]
    TrustZone(nova_tz::Info),
    File(file::Info),
}
