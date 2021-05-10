use bitfield::bitfield;
use bytes::Buf;

#[derive(Debug, PartialEq)]
pub enum KeyConfigType {
    Ecc,
    NotEcc,
}

impl From<u8> for KeyConfigType {
    fn from(v: u8) -> Self {
        match v & 4 == 4 {
            true => Self::Ecc,
            _ => Self::NotEcc,
        }
    }
}

impl From<KeyConfigType> for u8 {
    fn from(v: KeyConfigType) -> Self {
        match v {
            KeyConfigType::Ecc => 4,
            KeyConfigType::NotEcc => 7,
        }
    }
}

impl From<&[u8]> for KeyConfig {
    fn from(v: &[u8]) -> Self {
        let mut buf = v;
        Self(buf.get_u16())
    }
}

bitfield! {
    #[derive(PartialEq)]
    pub struct KeyConfig(u16);
    impl Debug;

    u8, auth_key, set_auth_key: 3, 0;
    intrusion_disable, set_intrusion_disable: 4;
    u8, x509_index, set_x509_index: 7, 6;

    private, set_private: 8;
    pub_info, set_pub_info: 9;
    u8, from into KeyConfigType, key_type, set_key_type: 12, 10;
    lockable, set_is_lockable: 13;
    req_random, set_req_random: 14;
    req_auth, set_req_auth: 15;
}

impl From<u16> for KeyConfig {
    fn from(v: u16) -> Self {
        Self(v)
    }
}

impl From<KeyConfig> for u16 {
    fn from(v: KeyConfig) -> Self {
        v.0
    }
}

impl From<&KeyConfig> for u16 {
    fn from(v: &KeyConfig) -> Self {
        v.0
    }
}

///  Returns a key configuration set up to store ECC key private keys.
impl Default for KeyConfig {
    fn default() -> Self {
        let mut result = KeyConfig(0);
        result.set_key_type(KeyConfigType::Ecc);
        result.set_is_lockable(true);
        result.set_private(true);
        result.set_pub_info(true);
        result
    }
}
