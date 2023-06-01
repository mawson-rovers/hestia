extern crate i2c_linux;

use std::io;

use byteorder::{ByteOrder, BigEndian, LittleEndian};
use i2c_linux::I2c;

use crate::I2cBus;

#[derive(Debug, Copy, Clone)]
pub struct I2cAddr(pub u8);

#[derive(Debug, Copy, Clone)]
pub struct I2cReg(pub u8);

pub(crate) trait I2cReadWrite {
    /// Read an unsigned 16-bit integer from the I2C device by address + register
    fn read_u16(&self, addr: I2cAddr, reg: I2cReg) -> io::Result<u16>;

    /// Write an unsigned 16-bit integer to the I2C device by address + register
    fn write_u16(&self, addr: I2cAddr, reg: I2cReg, data: u16) -> io::Result<()>;
}

#[derive(Debug, Clone, Copy)]
pub enum I2cDevice {
    BigEndian {
        bus: I2cBus,
    },
    LittleEndian {
        bus: I2cBus,
    },
}

impl I2cDevice {
    fn bus(&self) -> &I2cBus {
        match self {
            I2cDevice::BigEndian { bus } => bus,
            I2cDevice::LittleEndian { bus } => bus,
        }
    }

    fn i2c(&self) -> io::Result<I2c<std::fs::File>> {
        I2c::from_path(self.bus().path())
    }

    fn i2c_read_bytes<const LEN: usize>(&self, addr: I2cAddr, reg: I2cReg) -> io::Result<[u8; LEN]> {
        let mut data = [0; LEN];
        let mut i2c = self.i2c()?;
        // i2c.i2c_set_retries(0)?;
        // i2c.i2c_set_timeout(Duration::from_millis(10))?;  // doesn't actually work on the BBB :-(
        i2c.smbus_set_slave_address(addr.0 as u16, false)?;
        i2c.i2c_read_block_data(reg.0, &mut data)?;
        Ok(data)
    }

    fn i2c_write_bytes<const LEN: usize>(&self, addr: I2cAddr, reg: I2cReg, buf: &[u8; LEN])
                                         -> io::Result<()> {
        let mut i2c = self.i2c()?;
        // i2c.i2c_set_retries(0)?;
        // i2c.i2c_set_timeout(Duration::from_millis(10))?;
        i2c.smbus_set_slave_address(addr.0 as u16, false)?;
        i2c.i2c_write_block_data(reg.0, buf)
    }
}

impl std::fmt::Display for I2cDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "i2c{}", self.bus())
    }
}

impl I2cReadWrite for I2cDevice {
    fn read_u16(&self, addr: I2cAddr, reg: I2cReg) -> io::Result<u16> {
        let data: [u8; 2] = self.i2c_read_bytes::<2>(addr, reg)?;
        match self {
            I2cDevice::BigEndian { .. } => Ok(BigEndian::read_u16(&data)),
            I2cDevice::LittleEndian { .. } => Ok(LittleEndian::read_u16(&data)),
        }
    }

    fn write_u16(&self, addr: I2cAddr, reg: I2cReg, data: u16) -> io::Result<()> {
        let mut buf: [u8; 2] = [0; 2];
        match self {
            I2cDevice::BigEndian { .. } => BigEndian::write_u16(&mut buf, data),
            I2cDevice::LittleEndian { .. } => LittleEndian::write_u16(&mut buf, data),
        }
        self.i2c_write_bytes::<2>(addr, reg, &buf)
    }
}

/// Read a big-endian unsigned 16-bit integer from an I2C bus + address + register
pub fn i2c_read_u16_be(bus: I2cBus, addr: I2cAddr, reg: I2cReg) -> io::Result<u16> {
    I2cDevice::BigEndian { bus }.read_u16(addr, reg)
}

/// Read a little-endian unsigned 16-bit integer from an I2C bus + address + register
pub fn i2c_read_u16_le(bus: I2cBus, addr: I2cAddr, reg: I2cReg) -> io::Result<u16> {
    I2cDevice::LittleEndian { bus }.read_u16(addr, reg)
}
