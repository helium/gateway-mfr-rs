use crate::Result;
pub use gateway_security::device::{Device, DeviceArgs};
use helium_crypto::Keypair;
use serde::Serialize;

#[cfg(feature = "ecc608")]
mod ecc;
mod file;
#[cfg(feature = "nova-tz")]
mod nova_tz;
#[cfg(feature = "tpm")]
mod tpm;

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

pub trait GatewaySecurityDevice {
    fn get_config(&self) -> Result<Config>;
    fn provision(&self) -> Result<Keypair>;
    fn get_tests(&self) -> Vec<test::Test>;
}

impl GatewaySecurityDevice for Device {
    fn get_config(&self) -> Result<Config> {
        match self {
            #[cfg(feature = "ecc608")]
            Self::Ecc(device) => device.get_config(),
            #[cfg(feature = "tpm")]
            Self::Tpm(device) => device.get_config(),
            #[cfg(feature = "nova-tz")]
            Self::TrustZone(device) => device.get_config(),
            Self::File(device) => device.get_config(),
        }
    }

    fn provision(&self) -> Result<Keypair> {
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

    fn get_tests(&self) -> Vec<test::Test> {
        match self {
            #[cfg(feature = "ecc608")]
            Self::Ecc(device) => device.get_tests(),
            #[cfg(feature = "tpm")]
            Self::Tpm(device) => device.get_tests(),
            #[cfg(feature = "nova-tz")]
            Self::TrustZone(device) => device.get_tests(),
            Self::File(device) => device.get_tests(),
        }
    }
}
