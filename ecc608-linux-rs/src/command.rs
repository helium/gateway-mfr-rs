use crate::{
    constants::{
        ATCA_GENKEY, ATCA_INFO, ATCA_LOCK, ATCA_NONCE, ATCA_READ, ATCA_RSP_SIZE_MIN, ATCA_SIGN,
        ATCA_WRITE, CMD_STATUS_BYTE_COMM, CMD_STATUS_BYTE_ECC, CMD_STATUS_BYTE_EXEC,
        CMD_STATUS_BYTE_PARSE, CMD_STATUS_BYTE_SELF_TEST, CMD_STATUS_BYTE_SUCCESS,
        CMD_STATUS_BYTE_WATCHDOG,
    },
    Address, DataBuffer, Error, Result, Zone,
};
use bitfield::bitfield;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum KeyType {
    Public,
    Private,
}

impl From<&KeyType> for u8 {
    fn from(k: &KeyType) -> Self {
        match k {
            KeyType::Public => 0x00,
            KeyType::Private => 0x04,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum EccCommand {
    Info,
    GenKey { key_type: KeyType, slot: u8 },
    Read { is_32: bool, address: Address },
    Write { address: Address, data: Bytes },
    Lock { zone: Zone },
    Nonce { target: DataBuffer, data: Bytes },
    Sign { source: DataBuffer, key_slot: u8 },
}

bitfield! {
    #[derive(PartialEq)]
    struct ReadWriteParam(u8);
    impl Debug;
    is_32, set_is_32: 7;
    address_zone, set_address_zone: 1, 0;
}

impl From<ReadWriteParam> for u8 {
    fn from(v: ReadWriteParam) -> Self {
        v.0
    }
}

bitfield! {
    #[derive(PartialEq)]
    struct NonceParam(u8);
    impl Debug;
    u8, target, set_target: 7, 6;
    is_64, set_is_64: 5;
    u8, mode, set_mode: 1, 0;
}

impl From<NonceParam> for u8 {
    fn from(v: NonceParam) -> Self {
        v.0
    }
}

bitfield! {
    #[derive(PartialEq)]
    struct SignParam(u8);
    impl Debug;
    external, set_external: 7;
    u8, source, set_source: 5, 5;
}

impl From<SignParam> for u8 {
    fn from(v: SignParam) -> Self {
        v.0
    }
}

bitfield! {
    #[derive(PartialEq)]
    pub struct LockParam(u8);
    impl Debug;
    u8, zone, set_zone: 1, 0;
    u8, slot, set_slot: 5, 2;
    crc, set_crc: 7;
}

impl From<LockParam> for u8 {
    fn from(v: LockParam) -> Self {
        v.0
    }
}

#[derive(Debug, PartialEq)]
pub enum EccError {
    /// Command was properly received but the length, command opcode, or
    /// parameters are illegal regardless of the state (volatile and/or EEPROM
    /// configuration) of the ECC. Changes in the value of the command bits
    /// must be made before it is re-attempted.
    ParseError,
    /// A computation error occurred during ECC processing that caused the
    /// result to be invalid. Retrying the command may result in a successful
    /// execution.
    Fault,
    /// There was a self test error and the chip is in failure mode waiting for
    /// the failure to be cleared.
    SelfTestError,
    /// Command was properly received but could not be executed by the device in
    /// its current state. Changes in the device state or the value of the
    /// command bits must be made before it is re-attempted.
    ExecError,
    /// Command was not properly received by AT88SHA204 and should be
    /// re-transmitted by the I/O driver in the system. No attempt was made to
    /// parse or execute the command.
    CommsError,
    /// There is insufficient time to execute the given command before the
    /// watchdog timer will expire. The system must reset the watchdog timer by
    /// entering the idle or sleep modes.
    WatchDogError,
    /// Unknown or unhandled Ecc error
    Unknown(u8),
}

#[derive(Debug, PartialEq)]
pub enum EccResponse {
    Error(EccError),
    Data(Bytes),
}

macro_rules! put_cmd {
    ($dest:ident, $cmd:ident, $param1:expr, $param2:expr) => {
        $dest.put_u8($cmd);
        $dest.put_u8($param1);
        $dest.put_u16($param2);
    };
}

impl EccCommand {
    pub fn info() -> Self {
        Self::Info
    }

    pub fn genkey(key_type: KeyType, slot: u8) -> Self {
        Self::GenKey { key_type, slot }
    }

    pub fn read(is_32: bool, address: Address) -> Self {
        Self::Read { is_32, address }
    }

    pub fn write(address: Address, data: &[u8]) -> Self {
        Self::Write {
            address,
            data: Bytes::copy_from_slice(data),
        }
    }

    pub fn lock(zone: Zone) -> Self {
        Self::Lock { zone }
    }

    pub fn nonce(target: DataBuffer, data: Bytes) -> Self {
        Self::Nonce { target, data }
    }

    pub fn sign(source: DataBuffer, key_slot: u8) -> Self {
        Self::Sign { source, key_slot }
    }

    pub fn bytes_into(&self, bytes: &mut BytesMut) {
        bytes.put_slice(&[0x03, 0x00]);
        match self {
            Self::Info => {
                put_cmd!(bytes, ATCA_INFO, 0, 0);
            }
            Self::GenKey { key_type, slot } => {
                put_cmd!(bytes, ATCA_GENKEY, key_type.into(), *slot as u16);
            }
            Self::Read { is_32, address } => {
                let mut param1 = ReadWriteParam(0);
                param1.set_is_32(*is_32);
                param1.set_address_zone(address.zone());
                put_cmd!(bytes, ATCA_READ, param1.into(), u16::from(address));
            }
            Self::Write { address, data } => {
                let mut param1 = ReadWriteParam(0);
                param1.set_is_32(data.len() == 32);
                param1.set_address_zone(address.zone());
                put_cmd!(bytes, ATCA_WRITE, param1.into(), u16::from(address));
                bytes.extend_from_slice(data);
            }
            Self::Lock { zone } => {
                let mut param1 = LockParam(0);
                param1.set_crc(true);
                param1.set_zone(match zone {
                    Zone::Config => 0x00,
                    Zone::Data => 0x01,
                });
                put_cmd!(bytes, ATCA_LOCK, param1.into(), 0);
            }
            Self::Nonce { target, data } => {
                let mut param1 = NonceParam(0);
                param1.set_mode(0x03); // pass-through only for now
                param1.set_target(target.into());
                param1.set_is_64(data.len() == 64);
                put_cmd!(bytes, ATCA_NONCE, param1.into(), 0);
                bytes.extend_from_slice(data)
            }
            Self::Sign { source, key_slot } => {
                let mut param1 = SignParam(0);
                param1.set_source(source.into());
                param1.set_external(true);
                put_cmd!(bytes, ATCA_SIGN, param1.into(), *key_slot as u16);
            }
        }
        bytes[1] = (bytes.len() + 1) as u8;
        bytes.put_u16_le(crc(&bytes[1..]))
    }

    pub fn duration(&self) -> Duration {
        let micros = match self {
            Self::Info => 500,
            Self::GenKey { .. } => 59_000,
            Self::Read { .. } => 800,
            Self::Write { .. } => 8000,
            // ecc608b increases the default lock duration of 15_000 by about 30%
            Self::Lock { .. } => 19_500,
            Self::Nonce { .. } => 17_000,
            Self::Sign { .. } => 64_000,
        };
        Duration::from_micros(micros)
    }
}

impl EccResponse {
    pub fn from_bytes(buf: &[u8]) -> Result<Self> {
        if buf[0] == ATCA_RSP_SIZE_MIN {
            match buf[1] {
                CMD_STATUS_BYTE_SUCCESS => Ok(Self::Data(Bytes::new())),
                CMD_STATUS_BYTE_PARSE => Ok(Self::Error(EccError::ParseError)),
                CMD_STATUS_BYTE_ECC => Ok(Self::Error(EccError::Fault)),
                CMD_STATUS_BYTE_SELF_TEST => Ok(Self::Error(EccError::SelfTestError)),
                CMD_STATUS_BYTE_EXEC => Ok(Self::Error(EccError::ExecError)),
                CMD_STATUS_BYTE_COMM => Ok(Self::Error(EccError::CommsError)),
                CMD_STATUS_BYTE_WATCHDOG => Ok(Self::Error(EccError::WatchDogError)),
                error => Ok(Self::Error(EccError::Unknown(error))),
            }
        } else {
            let (buf, mut buf_crc) = buf.split_at(buf.len() - 2);
            let expected = crc(&buf);
            let actual = buf_crc.get_u16_le();
            if expected != actual {
                return Err(Error::crc(expected, actual));
            }
            Ok(Self::Data(Bytes::copy_from_slice(&buf[1..])))
        }
    }
}

impl EccError {
    pub fn is_recoverable(&self) -> bool {
        !matches!(self, Self::ParseError | Self::ExecError)
    }
}

fn crc(src: &[u8]) -> u16 {
    const POLYNOM: u16 = 0x8005;
    let mut crc: u16 = 0x0000;
    let mut data_bit;
    let mut crc_bit;
    for d in src {
        for b in 0..8 {
            if (d & 1 << b) == 0 {
                data_bit = 0;
            } else {
                data_bit = 1;
            }
            crc_bit = crc >> 15 & 0xff;
            crc <<= 1 & 0xffff;
            if data_bit != crc_bit {
                crc ^= POLYNOM;
            }
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info() {
        let packet = EccCommand::info();
        let mut bytes = BytesMut::with_capacity(ATCA_CMD_SIZE_MAX as usize);
        packet.bytes_into(&mut bytes);
        assert_eq!(
            &[0x03, 0x07, 0x30, 0x00, 0x00, 0x00, 0x03, 0x5D],
            &bytes[..]
        )
    }
}
