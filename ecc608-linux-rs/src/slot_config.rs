use bitfield::bitfield;
use bytes::Buf;

bitfield! {
    pub struct ReadKey(u8);
    impl Debug;
    pub external_signatures, set_external_signatures: 0;
    pub internal_signatures, set_internal_signatures: 1;
    pub ecdh_operation, set_ecdh_operation: 2;
    pub ecdh_write_slot, set_ecdh_write_slot: 3;
}

impl From<u8> for ReadKey {
    fn from(v: u8) -> Self {
        Self(v)
    }
}

impl From<ReadKey> for u8 {
    fn from(v: ReadKey) -> Self {
        v.0
    }
}

impl Default for ReadKey {
    fn default() -> Self {
        let mut result = Self(0);
        result.set_internal_signatures(true);
        result.set_external_signatures(true);
        result.set_ecdh_operation(true);
        result
    }
}

/// Write cofiguration from the write_config slot bits for a given command. The
/// interpretation of the write_config bits differs based on the command used.
#[derive(Debug, PartialEq)]
pub enum WriteConfig {
    Write(_WriteConfig),
    DeriveKey(DeriveKeyConfig),
    GenKey(GenKeyConfig),
    PrivWrite(PrivWriteConfig),
}

#[derive(Debug, PartialEq)]
pub enum WriteCommand {
    Write,
    DeriveKey,
    GenKey,
    PrivWrite,
}

impl WriteConfig {
    pub fn from(cmd: WriteCommand, v: u8) -> Self {
        match cmd {
            WriteCommand::Write => Self::Write(v.into()),
            WriteCommand::DeriveKey => Self::DeriveKey(v.into()),
            WriteCommand::GenKey => Self::GenKey(v.into()),
            WriteCommand::PrivWrite => Self::PrivWrite(v.into()),
        }
    }
}

impl From<WriteConfig> for u8 {
    fn from(v: WriteConfig) -> Self {
        match v {
            WriteConfig::Write(cfg) => cfg.into(),
            WriteConfig::DeriveKey(cfg) => cfg.into(),
            WriteConfig::GenKey(cfg) => cfg.into(),
            WriteConfig::PrivWrite(cfg) => cfg.into(),
        }
    }
}

impl Default for WriteConfig {
    fn default() -> Self {
        WriteConfig::GenKey(GenKeyConfig::Valid)
    }
}

#[derive(Debug, PartialEq)]
pub enum _WriteConfig {
    /// Clear text writes are always permitted on this slot. Slots set to
    /// alwaysshould never be used as key storage. Either 4 or 32 bytes may
    /// bewritten to this slot
    Always,
    /// If a validated public key is stored in the slot, writes are prohibited.
    /// UseVerify(Invalidate) to invalidate prior to writing. Do not use
    /// thismode unless the slot contains a public key.
    PubInValid,
    /// Writes are never permitted on this slot using the Write command.Slots
    /// set to never can still be used as key storage.
    Never,
    /// Writes to this slot require a properly computed MAC, and the inputdata
    /// must be encrypted by the system with WriteKey using theencryption
    /// algorithm documented in the Write command description(Section Write
    /// Command). 4 byte writes to this slot are prohibited.
    Encrypt,
}

impl From<u8> for _WriteConfig {
    fn from(v: u8) -> Self {
        match v {
            0 => _WriteConfig::Always,
            1 => _WriteConfig::PubInValid,
            _ if v >> 1 == 1 => _WriteConfig::Never,
            _ if v >> 2 == 2 => _WriteConfig::Never,
            _ if v & 4 == 4 => _WriteConfig::Encrypt,
            _ => panic!("invalid write config {:?}", v),
        }
    }
}

impl From<_WriteConfig> for u8 {
    fn from(v: _WriteConfig) -> Self {
        match v {
            _WriteConfig::Always => 0,
            _WriteConfig::PubInValid => 1,
            _WriteConfig::Never => 2,
            _WriteConfig::Encrypt => 4,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum DeriveKeyConfig {
    ///  DeriveKey command can be run with/without authorizing MAC. Source Key:
    /// Target
    Roll(bool),
    /// DeriveKey command can be run with/without authorizing MAC. Source Key:
    /// Parent
    Create(bool),
    /// Slots with this write configutation can not be used as the target of a
    /// DeriveKey.
    Invalid,
}

impl From<u8> for DeriveKeyConfig {
    fn from(v: u8) -> Self {
        match v & 11 {
            2 => Self::Roll(false),
            10 => Self::Roll(true),
            3 => Self::Create(false),
            11 => Self::Create(true),
            _ => Self::Invalid,
        }
    }
}

impl From<DeriveKeyConfig> for u8 {
    fn from(v: DeriveKeyConfig) -> Self {
        match v {
            DeriveKeyConfig::Roll(false) => 2,
            DeriveKeyConfig::Roll(true) => 10,
            DeriveKeyConfig::Create(false) => 3,
            DeriveKeyConfig::Create(true) => 11,
            DeriveKeyConfig::Invalid => 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GenKeyConfig {
    /// GenKey may not be used to write random keys into this slot.
    Valid,
    /// GenKey may be used to write random keys into this slot.
    Invalid,
}

impl From<u8> for GenKeyConfig {
    fn from(v: u8) -> Self {
        match v & 2 == 0 {
            true => Self::Invalid,
            _ => Self::Valid,
        }
    }
}

impl From<GenKeyConfig> for u8 {
    fn from(v: GenKeyConfig) -> Self {
        match v {
            GenKeyConfig::Invalid => 0,
            GenKeyConfig::Valid => 2,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PrivWriteConfig {
    /// PrivWrite will return an error if the target key slot has this value.
    Invalid,
    /// Writes to this slot require a properly computed MAC and the inputdata
    /// must be encrypted by the system with SlotConfig.WriteKey using the
    /// encryption algorithm documented with PrivWrite.
    Encrypt,
}

impl From<u8> for PrivWriteConfig {
    fn from(v: u8) -> Self {
        match v & 4 == 0 {
            true => Self::Invalid,
            _ => Self::Encrypt,
        }
    }
}

impl From<PrivWriteConfig> for u8 {
    fn from(v: PrivWriteConfig) -> Self {
        match v {
            PrivWriteConfig::Invalid => 0,
            PrivWriteConfig::Encrypt => 4,
        }
    }
}

bitfield! {
    #[derive(PartialEq)]
    pub struct SlotConfig(u16);
    impl Debug;
    pub secret, set_secret: 15;
    pub encrypt_read, set_encrypt_read: 14;
    pub limited_use, set_limited_use: 13;
    pub no_mac, set_no_mac: 12;
    u8, from into ReadKey, read_key, set_read_key: 11, 8;
    u8, _write_config, _set_write_config: 7, 4;
    u8, write_key, set_write_key: 3, 0;
}

impl From<&[u8]> for SlotConfig {
    fn from(v: &[u8]) -> Self {
        let mut buf = v;
        Self(buf.get_u16())
    }
}

impl From<u16> for SlotConfig {
    fn from(v: u16) -> Self {
        Self(v)
    }
}

impl From<SlotConfig> for u16 {
    fn from(v: SlotConfig) -> Self {
        v.0
    }
}

impl From<&SlotConfig> for u16 {
    fn from(v: &SlotConfig) -> Self {
        v.0
    }
}

/// A convenience function to get a slot configuratoin set up to
/// generate and store ECDSA private keys.
impl Default for SlotConfig {
    fn default() -> Self {
        let mut result = SlotConfig(0);
        result.set_write_config(WriteConfig::default());
        result.set_write_key(0);
        result.set_secret(true);
        result.set_encrypt_read(false);
        result.set_limited_use(false);
        result.set_no_mac(true);
        result.set_read_key(ReadKey::default());
        result
    }
}

impl SlotConfig {
    pub fn write_config(&self, cmd: WriteCommand) -> WriteConfig {
        WriteConfig::from(cmd, self._write_config())
    }

    pub fn set_write_config<C>(&mut self, config: C)
    where
        C: Into<u8>,
    {
        self._set_write_config(config.into())
    }
}
