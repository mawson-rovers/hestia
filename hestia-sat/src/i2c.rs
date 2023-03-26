extern crate i2c_linux;

use std::io;

use byteorder::ByteOrder;
use i2c_linux::I2c;
use crate::I2cBus;

#[derive(Debug, Copy, Clone)]
pub struct I2cAddr(pub u8);

#[derive(Debug, Copy, Clone)]
pub struct I2cReg(pub u8);

pub fn i2c_read_u16<Order: ByteOrder>(bus: &I2cBus, addr: I2cAddr, reg: I2cReg) -> io::Result<u16> {
    let data: [u8; 2] = i2c_read_bytes::<2>(bus, addr, reg)?;
    Ok(Order::read_u16(&data))
}

pub fn i2c_read_i16<Order: ByteOrder>(bus: &I2cBus, addr: I2cAddr, reg: I2cReg) -> io::Result<i16> {
    let data: [u8; 2] = i2c_read_bytes::<2>(bus, addr, reg)?;
    Ok(Order::read_i16(&data))
}

fn i2c_read_bytes<const LEN: usize>(bus: &I2cBus, addr: I2cAddr, reg: I2cReg) -> io::Result<[u8; LEN]> {
    let mut data = [0; LEN];
    let mut i2c = I2c::from_path(bus.path)?;
    i2c.smbus_set_slave_address(addr.0 as u16, false)?;
    i2c.i2c_read_block_data(reg.0, &mut data)?;
    Ok(data)
}

pub fn i2c_write_u16<Order: ByteOrder>(bus: &I2cBus, addr: I2cAddr, reg: I2cReg, data: u16) -> io::Result<()> {
    let mut buf: [u8; 2] = [0; 2];
    Order::write_u16(&mut buf, data);
    let mut i2c = I2c::from_path(bus.path)?;
    i2c.smbus_set_slave_address(addr.0 as u16, false)?;
    i2c.i2c_write_block_data(reg.0, &buf)
}
