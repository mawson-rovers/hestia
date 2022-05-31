#!/usr/bin/env python3

import math
import time

from smbus2 import SMBus

# constants from Adafruit CPP example
# https://github.com/adafruit/Adafruit_MCP9808_Library
MCP9808_I2CADDR_DEFAULT = 0x18  # I2C address
MCP9808_REG_CONFIG = 0x01  # MCP9808 config register

MCP9808_REG_CONFIG_SHUTDOWN = 0x0100  # shutdown config
MCP9808_REG_CONFIG_CRITLOCKED = 0x0080  # critical trip lock
MCP9808_REG_CONFIG_WINLOCKED = 0x0040  # alarm window lock
MCP9808_REG_CONFIG_INTCLR = 0x0020  # interrupt clear
MCP9808_REG_CONFIG_ALERTSTAT = 0x0010  # alert output status
MCP9808_REG_CONFIG_ALERTCTRL = 0x0008  # alert output control
MCP9808_REG_CONFIG_ALERTSEL = 0x0004  # alert output select
MCP9808_REG_CONFIG_ALERTPOL = 0x0002  # alert output polarity
MCP9808_REG_CONFIG_ALERTMODE = 0x0001  # alert output mode

MCP9808_REG_UPPER_TEMP = 0x02  # upper alert boundary
MCP9808_REG_LOWER_TEMP = 0x03  # lower alert boundary
MCP9808_REG_CRIT_TEMP = 0x04  # critical temperature
MCP9808_REG_AMBIENT_TEMP = 0x05  # ambient temperature
MCP9808_REG_MANUF_ID = 0x06  # manufacturer ID
MCP9808_REG_DEVICE_ID = 0x07  # device ID
MCP9808_REG_RESOLUTION = 0x08  # resolution


def read_int(bus, reg):
    return int.from_bytes(bus.read_i2c_block_data(
        MCP9808_I2CADDR_DEFAULT, reg, 2), "big")


def write_int(bus, reg, val):
    bus.write_i2c_block_data(MCP9808_I2CADDR_DEFAULT, reg,
                             list(int.to_bytes(val, 2, "big")))


def init(bus):
    write_int(bus, MCP9808_REG_CONFIG, 0x0)

    bus.pec = 1  # Enable packet error correction
    manuf_id = read_int(bus, MCP9808_REG_MANUF_ID)
    if manuf_id != 0x0054:
        raise ValueError("Incorrect manufacturer ID", manuf_id)
    device_id = read_int(bus, MCP9808_REG_DEVICE_ID)
    if device_id != 0x0400:
        raise ValueError("Incorrect device ID", device_id)


def read_temp_celsius(bus):
    # logic borrowed from Adafruit CPP example
    # https://github.com/adafruit/Adafruit_MCP9808_Library
    t = read_int(bus, MCP9808_REG_AMBIENT_TEMP)
    if t == 0xffff:
        return math.nan

    temp = t & 0x0fff
    temp = temp / 16.0
    if t & 0x1000:
        temp = temp - 256
    return temp


def main():
    with SMBus(1) as bus:
        init(bus)
        while True:
            temp = read_temp_celsius(bus)
            if math.isnan(temp):
                print("Error reading temperature")
            else:
                print(str(temp) + "°C")
            time.sleep(1)


if __name__ == "__main__":
    main()
