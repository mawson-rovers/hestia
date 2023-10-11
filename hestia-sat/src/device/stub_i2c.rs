use std::io;
use byteorder::{BigEndian, LittleEndian, ByteOrder};
use crate::device::i2c::{I2cAddr, I2cBus, I2cReg};
use log::info;

impl I2cBus {
    pub fn path(&self) -> String {
        format!("i2c-stub-{}", self.id)
    }

    pub fn exists(&self) -> bool {
        true
    }

    pub fn read_bytes<const LEN: usize>(&self, addr: I2cAddr, reg: I2cReg) -> io::Result<[u8; LEN]> {
        let mut data = [0; LEN];
        match addr {
            I2cAddr(0x08) => { // MSP430
                match reg {
                    I2cReg(0x10) => { // version
                        LittleEndian::write_u16(&mut data, 220); // v2.2
                    },
                    I2cReg(0x11) => { // flags
                        LittleEndian::write_u16(&mut data, 1); // OK
                    },
                    I2cReg(0x20) => { // heater mode
                        LittleEndian::write_u16(&mut data, 0); // OFF
                    },
                    I2cReg(0x22) => { // target sensor
                        LittleEndian::write_u16(&mut data, 0); // TH1
                    },
                    I2cReg(0x23) => { // duty cycle
                        LittleEndian::write_u16(&mut data, 255);
                    },
                    _ => {            // ADC sensors
                        LittleEndian::write_u16(&mut data, 2048 + addr.0 as u16);
                    },
                }
            },
            I2cAddr(0x4A) => { // ADS7828 ADC
                BigEndian::write_u16(&mut data, 2048 + addr.0 as u16);
            },
            I2cAddr(0x48) |
            I2cAddr(0x4F) |
            I2cAddr(0x49) |
            I2cAddr(0x4B) => { // MAX31725 I2C temp sensors
                let temp: u16 = 25 << 8;
                let frac: u16 = (addr.0 as u16) << 1;
                BigEndian::write_u16(&mut data, temp + frac);
            },
            _ => {},
        }
        Ok(data)
    }

    pub fn write_bytes<const LEN: usize>(&self, addr: I2cAddr, reg: I2cReg, buf: &[u8; LEN])
                                     -> io::Result<()> {
        info!("Writing {} bytes to I2C{} addr/reg {}, {}: {:02x?}",
            buf.len(), self.id, addr, reg, buf);
        // don't do anything - just discard the data
        Ok(())
    }
}


