use crate::constants::{ATCA_CMD_SIZE_MAX, WAKE_DELAY};
use crate::{
    command::{EccCommand, EccResponse},
    Address, DataBuffer, Error, KeyConfig, Result, SlotConfig, Zone,
};
use bytes::{BufMut, Bytes, BytesMut};
use i2c_linux::{I2c, ReadFlags};
use std::{fs::File, thread, time::Duration};

pub use crate::command::KeyType;

pub struct Ecc {
    i2c: I2c<File>,
    address: u16,
}

pub(crate) const RECV_RETRIES: u8 = 2;
pub(crate) const RECV_RETRY_WAIT: Duration = Duration::from_millis(50);
pub(crate) const CMD_RETRIES: u8 = 10;

impl Ecc {
    pub fn from_path(path: &str, address: u16) -> Result<Self> {
        let mut i2c = I2c::from_path(path)?;
        i2c.smbus_set_slave_address(address, false)?;
        Ok(Self { i2c, address })
    }

    pub fn get_info(&mut self) -> Result<Bytes> {
        self.send_command(&EccCommand::info())
    }

    /// Returns the 9 bytes that represent the serial number of the ECC. Per
    /// section 2.2.6 of the Data Sheet the first two, and last byte of the
    /// returned binary will always be `[0x01, 0x23]` and `0xEE`
    pub fn get_serial(&mut self) -> Result<Bytes> {
        let bytes = self.read(true, &Address::config(0, 0)?)?;
        let mut result = BytesMut::with_capacity(9);
        result.extend_from_slice(&bytes.slice(0..=3));
        result.extend_from_slice(&bytes.slice(8..=12));
        Ok(result.freeze())
    }

    pub fn genkey(&mut self, key_type: KeyType, slot: u8) -> Result<Bytes> {
        self.send_command(&EccCommand::genkey(key_type, slot))
    }

    pub fn get_slot_config(&mut self, slot: u8) -> Result<SlotConfig> {
        let bytes = self.read(false, &Address::slot_config(slot)?)?;
        let (s0, s1) = bytes.split_at(2);
        match slot & 1 == 0 {
            true => Ok(SlotConfig::from(s0)),
            false => Ok(SlotConfig::from(s1)),
        }
    }

    pub fn set_slot_config(&mut self, slot: u8, config: &SlotConfig) -> Result {
        let slot_address = Address::slot_config(slot)?;
        let bytes = self.read(false, &slot_address)?;
        let (s0, s1) = bytes.split_at(2);
        let mut new_bytes = BytesMut::with_capacity(4);
        match slot & 1 == 0 {
            true => {
                new_bytes.put_u16(config.into());
                new_bytes.extend_from_slice(s1);
            }
            false => {
                new_bytes.extend_from_slice(s0);
                new_bytes.put_u16(config.into());
            }
        }
        self.write(&slot_address, &new_bytes.freeze())
    }

    pub fn get_key_config(&mut self, slot: u8) -> Result<KeyConfig> {
        let bytes = self.read(false, &Address::key_config(slot)?)?;
        let (s0, s1) = bytes.split_at(2);
        match slot & 1 == 0 {
            true => Ok(KeyConfig::from(s0)),
            false => Ok(KeyConfig::from(s1)),
        }
    }

    pub fn set_key_config(&mut self, slot: u8, config: &KeyConfig) -> Result {
        let slot_address = Address::key_config(slot)?;
        let bytes = self.read(false, &slot_address)?;
        let (s0, s1) = bytes.split_at(2);
        let mut new_bytes = BytesMut::with_capacity(4);
        match slot & 1 == 0 {
            true => {
                new_bytes.put_u16(config.into());
                new_bytes.extend_from_slice(s1);
            }
            false => {
                new_bytes.extend_from_slice(s0);
                new_bytes.put_u16(config.into());
            }
        }
        self.write(&slot_address, &new_bytes.freeze())
    }

    pub fn get_locked(&mut self, zone: Zone) -> Result<bool> {
        let bytes = self.read(false, &Address::config(2, 5)?)?;
        let (_, s1) = bytes.split_at(2);
        match zone {
            Zone::Config => Ok(s1[1] == 0),
            Zone::Data => Ok(s1[0] == 0),
        }
    }

    pub fn set_locked(&mut self, zone: Zone) -> Result {
        self.send_command(&EccCommand::lock(zone)).map(|_| ())
    }

    pub fn sign(&mut self, key_slot: u8, data: &[u8]) -> Result<Bytes> {
        self.nonce(DataBuffer::MessageDigest, data)?;
        self.send_command(&EccCommand::sign(DataBuffer::MessageDigest, key_slot))
    }

    pub fn nonce(&mut self, target: DataBuffer, data: &[u8]) -> Result {
        self.send_command(&EccCommand::nonce(target, Bytes::copy_from_slice(data)))
            .map(|_| ())
    }

    pub fn read(&mut self, read_32: bool, address: &Address) -> Result<Bytes> {
        self.send_command(&EccCommand::read(read_32, address.clone()))
    }

    pub fn write(&mut self, address: &Address, bytes: &[u8]) -> Result {
        self.send_command(&EccCommand::write(address.clone(), bytes))
            .map(|_| ())
    }

    fn send_wake(&mut self) {
        let _ = self.send_buf(&[0]);
    }

    fn send_sleep(&mut self) {
        let _ = self.send_buf(&[1]);
    }

    pub(crate) fn send_command(&mut self, command: &EccCommand) -> Result<Bytes> {
        self.send_command_retries(command, CMD_RETRIES)
    }

    pub(crate) fn send_command_retries(
        &mut self,
        command: &EccCommand,
        retries: u8,
    ) -> Result<Bytes> {
        let mut buf = BytesMut::with_capacity(ATCA_CMD_SIZE_MAX as usize);
        for retry in 1..retries {
            buf.clear();
            command.bytes_into(&mut buf);

            self.send_wake();
            thread::sleep(WAKE_DELAY);

            if let Err(_err) = self.send_recv_buf(command.duration(), &mut buf) {
                if retry == retries {
                    break;
                } else {
                    continue;
                }
            }

            let response = EccResponse::from_bytes(&buf[..])?;
            self.send_sleep();
            match response {
                EccResponse::Data(bytes) => return Ok(bytes),
                EccResponse::Error(err) if err.is_recoverable() && retry < retries => {
                    continue;
                }
                EccResponse::Error(err) => return Err(Error::ecc(err)),
            }
        }
        Err(Error::timeout())
    }

    fn send_recv_buf(&mut self, delay: Duration, buf: &mut BytesMut) -> Result {
        self.send_buf(&buf[..])?;
        thread::sleep(delay);
        self.recv_buf(buf)
    }

    pub(crate) fn send_buf(&mut self, buf: &[u8]) -> Result {
        let write_msg = i2c_linux::Message::Write {
            address: self.address,
            data: &buf,
            flags: Default::default(),
        };

        self.i2c.i2c_transfer(&mut [write_msg])?;
        Ok(())
    }

    pub(crate) fn recv_buf(&mut self, buf: &mut BytesMut) -> Result {
        unsafe { buf.set_len(1) };
        buf[0] = 0xff;
        for _retry in 0..RECV_RETRIES {
            let msg = i2c_linux::Message::Read {
                address: self.address,
                data: &mut buf[0..1],
                flags: Default::default(),
            };
            if let Err(_err) = self.i2c.i2c_transfer(&mut [msg]) {
            } else {
                break;
            }
            thread::sleep(RECV_RETRY_WAIT);
        }
        let count = buf[0] as usize;
        if count == 0xff {
            return Err(Error::timeout());
        }
        unsafe { buf.set_len(count) };
        let read_msg = i2c_linux::Message::Read {
            address: self.address,
            data: &mut buf[1..count],
            flags: ReadFlags::NO_START,
        };
        self.i2c.i2c_transfer(&mut [read_msg])?;
        Ok(())
    }
}
