use crate::{anyhow, Result};
use helium_crypto::Keypair;
use http::Uri;
use serde::Serialize;
use std::str::FromStr;

mod ecc;

/// A security device to work with. Security devices come in all forms. This
/// abstracts them into one with a well defined interface for doing what this
/// tool needs to do with them.
#[derive(Debug)]
pub enum Device {
    Ecc(ecc::Device),
}

/// Represents the configuration state for the given security device. This
/// information should include enpugh detail to convey that the security device
/// is "locked" so key material can be written to it.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Config {
    Ecc(ecc::Config),
}

pub mod test {
    use crate::{device::ecc, Result};
    use serde::Serialize;
    use std::{collections::HashMap, fmt};

    /// Represents a single test for a given device
    pub enum Test {
        Ecc(ecc::Test),
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
                Self::Ecc(test) => test.run(),
            }
        }
    }

    impl fmt::Display for Test {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Ecc(test) => test.fmt(f),
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
            .map_err(|err| anyhow!("invalid device url \"{}\": {:?}", s, err))?;
        match url.scheme_str() {
            Some("ecc") => Ok(Self::Ecc(ecc::Device::from_url(&url)?)),
            _ => Err(anyhow!("invalid device url \"{}\"", s)),
        }
    }
}

impl Device {
    pub fn get_info(&self) -> Result<Info> {
        let info = match self {
            Self::Ecc(device) => Info::Ecc(device.get_info()?),
        };
        Ok(info)
    }

    pub fn get_config(&self) -> Result<Config> {
        let config = match self {
            Self::Ecc(device) => Config::Ecc(device.get_config()?),
        };
        Ok(config)
    }

    pub fn get_keypair(&self, create: bool) -> Result<Keypair> {
        let keypair = match self {
            Self::Ecc(device) => device.get_keypair(create)?,
        };
        Ok(keypair)
    }

    pub fn provision(&self) -> Result<Keypair> {
        let keypair = match self {
            Self::Ecc(device) => device.provision()?,
        };
        Ok(keypair)
    }

    pub fn get_tests(&self) -> Vec<test::Test> {
        match self {
            Self::Ecc(device) => device
                .get_tests()
                .into_iter()
                .map(test::Test::Ecc)
                .collect(),
        }
    }
}

/// Represents useful device information like model and serial number.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Info {
    Ecc(ecc::Info),
}
