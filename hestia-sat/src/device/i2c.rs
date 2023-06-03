extern crate i2c_linux;

use std::io;
use std::path::Path;

use byteorder::ByteOrder;
use i2c_linux::I2c;
use log::{debug, info, warn};

#[derive(Debug, Copy, Clone)]
pub struct I2cAddr(pub u8);

impl std::fmt::Display for I2cAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:02x}", self.0)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct I2cReg(pub u8);

impl std::fmt::Display for I2cReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:02x}", self.0)
    }
}

impl From<I2cAddr> for I2cReg {
    fn from(addr: I2cAddr) -> Self {
        I2cReg(addr.0)
    }
}

/// Byte-oriented interface to I2C bus
#[derive(Debug, Copy, Clone)]
pub struct I2cBus {
    pub id: u8,
}

impl From<u8> for I2cBus {
    fn from(id: u8) -> Self {
        I2cBus { id }
    }
}

impl I2cBus {
    pub fn path(&self) -> String {
        format!("/dev/i2c-{}", self.id)
    }

    pub fn exists(&self) -> bool {
        Path::new(&self.path()).exists()
    }

    fn open_bus(&self) -> io::Result<I2c<std::fs::File>> {
        I2c::from_path(&self.path())
    }

    fn read_bytes<const LEN: usize>(&self, addr: I2cAddr, reg: I2cReg) -> io::Result<[u8; LEN]> {
        let mut data = [0; LEN];
        let mut i2c = self.open_bus()?;
        // i2c.i2c_set_retries(0)?;
        // i2c.i2c_set_timeout(Duration::from_millis(10))?;  // doesn't actually work on the BBB :-(
        i2c.smbus_set_slave_address(addr.0 as u16, false)?;
        i2c.i2c_read_block_data(reg.0, &mut data)?;
        Ok(data)
    }

    fn write_bytes<const LEN: usize>(&self, addr: I2cAddr, reg: I2cReg, buf: &[u8; LEN])
                                     -> io::Result<()> {
        let mut i2c = self.open_bus()?;
        // i2c.i2c_set_retries(0)?;
        // i2c.i2c_set_timeout(Duration::from_millis(10))?;
        i2c.smbus_set_slave_address(addr.0 as u16, false)?;
        i2c.i2c_write_block_data(reg.0, buf)
    }
}

impl std::fmt::Display for I2cBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Clone)]
pub enum I2cByteOrder {
    LittleEndian,
    BigEndian,
}

trait ByteConverter {
    fn read_u16(&self, buf: &[u8]) -> u16;
    fn write_u16(&self, buf: &mut [u8], data: u16);
}

impl ByteConverter for I2cByteOrder {
    /// Read an unsigned 16-bit integer from the I2C device by address + register
    fn read_u16(&self, buf: &[u8]) -> u16 {
        match self {
            I2cByteOrder::LittleEndian => byteorder::LittleEndian::read_u16(buf),
            I2cByteOrder::BigEndian => byteorder::BigEndian::read_u16(buf),
        }
    }

    /// Write an unsigned 16-bit integer to the I2C device by address + register
    fn write_u16(&self, buf: &mut [u8], data: u16) {
        match self {
            I2cByteOrder::LittleEndian => byteorder::LittleEndian::write_u16(buf, data),
            I2cByteOrder::BigEndian => byteorder::BigEndian::write_u16(buf, data),
        }
    }
}

/// I2C device which can read/write u16 data
#[derive(Debug, Clone)]
pub struct I2cDevice {
    bus: I2cBus,
    addr: I2cAddr,
    byte_order: I2cByteOrder,
}

impl I2cDevice {
    /// Read little-endian unsigned 16-bit integers from an I2C bus + addr
    pub fn little_endian(bus: I2cBus, addr: I2cAddr) -> Self {
        I2cDevice { bus, byte_order: I2cByteOrder::LittleEndian, addr }
    }

    /// Read big-endian unsigned 16-bit integers from an I2C bus + addr
    pub fn big_endian(bus: I2cBus, addr: I2cAddr) -> Self {
        I2cDevice { bus, byte_order: I2cByteOrder::BigEndian, addr }
    }

    pub fn read_u16(&self, reg: I2cReg) -> io::Result<u16> {
        let data: [u8; 2] = self.bus.read_bytes::<2>(self.addr, reg)?;
        Ok(self.byte_order.read_u16(&data))
    }

    pub fn write_u16(&self, reg: I2cReg, data: u16) -> io::Result<()> {
        let mut buf: [u8; 2] = [0; 2];
        self.byte_order.write_u16(&mut buf, data);
        self.bus.write_bytes::<2>(self.addr, reg, &buf)
    }
}


/// Wrapper around I2cDevice that provides logging about what is going on, with a read/write API
/// for named u16 registers.
#[derive(Debug, Clone)]
pub struct LoggingI2cDevice {
    name: String,
    device: I2cDevice,
}

impl std::fmt::Display for LoggingI2cDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "i2c-{}/{}", self.device.bus, self.name)
    }
}

impl LoggingI2cDevice {
    pub fn new(name: String, device: I2cDevice) -> LoggingI2cDevice {
        LoggingI2cDevice { name, device }
    }

    pub fn read_register(&self, reg: I2cReg, desc: &str) -> crate::ReadResult<u16> {
        info!("{}: Reading {} from addr {}, reg {}",
            self, desc, self.device.addr, reg);
        match self.device.read_u16(reg) {
            Ok(result) => {
                debug!("{}: Read value <{}> from {}", self, result, desc);
                Ok(result)
            },
            Err(e) => {
                warn!("{}: Could not read from {}: {:?}", self, desc, e);
                Err(e.into())
            },
        }
    }

    /// Writes a value to the I2C register on the device. Logs a warning if it fails,
    /// debug if it succeeds.
    pub fn write_register(&self, reg: I2cReg, desc: &str, data: u16) -> () {
        info!("{}: Setting {} to value <{}> (addr {}, reg {})",
            self.name, desc, self.device.addr, reg, data);
        match self.device.write_u16(reg, data) {
            Ok(_) => {
                debug!("{}: Set {} to value <{}>", self, desc, data);
            },
            Err(e) => {
                warn!("{}: Failed to set {}: {:?}", self, desc, e);
            }
        }
    }
}

