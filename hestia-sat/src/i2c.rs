extern crate i2c_linux;

use std::io;

use byteorder::{ByteOrder, BigEndian, LittleEndian};
use i2c_linux::I2c;

use crate::I2cBus;

#[derive(Debug, Copy, Clone)]
pub struct I2cAddr(pub u8);

#[derive(Debug, Copy, Clone)]
pub struct I2cReg(pub u8);

/// Read a big-endian unsigned 16-bit integer from an I2C bus + address + register
pub fn i2c_read_u16_be(bus: &I2cBus, addr: I2cAddr, reg: I2cReg) -> io::Result<u16> {
    let data: [u8; 2] = i2c_read_bytes::<2>(bus, addr, reg)?;
    Ok(BigEndian::read_u16(&data))
}

/// Read a little-endian unsigned 16-bit integer from an I2C bus + address + register
pub fn i2c_read_u16_le(bus: &I2cBus, addr: I2cAddr, reg: I2cReg) -> io::Result<u16> {
    let data: [u8; 2] = i2c_read_bytes::<2>(bus, addr, reg)?;
    Ok(LittleEndian::read_u16(&data))
}

fn i2c_read_bytes<const LEN: usize>(bus: &I2cBus, addr: I2cAddr, reg: I2cReg) -> io::Result<[u8; LEN]> {
    let mut data = [0; LEN];
    let mut i2c = I2c::from_path(bus.path())?;
    // i2c.i2c_set_retries(0)?;
    // i2c.i2c_set_timeout(Duration::from_millis(10))?;
    i2c.smbus_set_slave_address(addr.0 as u16, false)?;
    i2c.i2c_read_block_data(reg.0, &mut data)?;
    Ok(data)
}

/// Write a little-endian unsigned 16-bit integer to an I2C bus + address + register
pub fn i2c_write_u16_le(bus: &I2cBus, addr: I2cAddr, reg: I2cReg, data: u16) -> io::Result<()> {
    let mut buf: [u8; 2] = [0; 2];
    LittleEndian::write_u16(&mut buf, data);
    let mut i2c = I2c::from_path(bus.path())?;
    // i2c.i2c_set_retries(0)?;
    // i2c.i2c_set_timeout(Duration::from_millis(10))?;
    i2c.smbus_set_slave_address(addr.0 as u16, false)?;
    i2c.i2c_write_block_data(reg.0, &buf)
}
