"""
This is an internal module. See :meth:`hestia.board.Hestia` for the public API.

Utility methods for I2C interface using smbus2
"""
from pathlib import Path

from smbus2 import SMBus

BEAGLE_I2C_BUS = 2


def i2c_read_int(addr, reg, byteorder="big", signed=False):
    with SMBus(BEAGLE_I2C_BUS) as bus:
        return int.from_bytes(bus.read_i2c_block_data(
            addr, reg, 2), byteorder=byteorder, signed=signed)


def i2c_write_int(addr, reg, val, byteorder="big", signed=False):
    with SMBus(BEAGLE_I2C_BUS) as bus:
        bus.write_i2c_block_data(addr, reg, list(
            int.to_bytes(val, 2, byteorder=byteorder, signed=signed)))


def device_exists() -> bool:
    return Path("/dev/i2c-{}".format(BEAGLE_I2C_BUS)).exists()
