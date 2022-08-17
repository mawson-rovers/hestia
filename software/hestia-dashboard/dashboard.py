#!/usr/bin/env python3
from dataclasses import dataclass
from decimal import Decimal

from nicegui import ui
from smbus2 import SMBus


@dataclass
class Sensor:
    id: str
    addr: int
    label: str


MSP430_I2C_ADDR = 0x08
PRIMARY_SENSORS = [
    Sensor("TH1", 0x01, "Centre"),
    Sensor("TH2", 0x02, "Top-left of heater"),
    Sensor("TH3", 0x03, "Bottom-right of heater"),
    Sensor("J7", 0x07, "Mounted"),
    Sensor("J8", 0x08, "Mounted"),
    Sensor("J9", 0x09, "Mounted"),
    Sensor("J10", 0x0A, "Mounted"),
    Sensor("J11", 0x0B, "Mounted"),
]

ADS7828_I2C_ADDR = 0x4A
SECONDARY_SENSORS = [
    Sensor("TH4", 0x00, "Centre"),
    Sensor("TH5", 0x01, "Top-right"),
    Sensor("TH6", 0x02, "Bottom-left of heater"),
    Sensor("J12", 0x03, "Mounted"),
    Sensor("J13", 0x04, "Mounted"),
    Sensor("J14", 0x05, "Mounted"),
    Sensor("J15", 0x06, "Mounted"),
    Sensor("J16", 0x07, "Mounted"),
]

TERTIARY_SENSORS = [
    Sensor("U4", 0x48, "Top-left"),
    Sensor("U5", 0x4F, "Top-right"),
    Sensor("U6", 0x49, "Bottom-right"),
    Sensor("U7", 0x4B, "Centre"),
]

MAX31725_REG_TEMP = 0x00
MAX31725_REG_CONFIG = 0x01
MAX31725_REG_THYST_LOW_TRIP = 0x02
MAX31725_REG_TOS_HIGH_TRIP = 0x03
MAX31725_REG_MAX = 0x03
MAX31725_CF_LSB = Decimal('0.00390625')

BEAGLE_I2C_BUS = 2


def read_int(bus, addr, reg, signed=False):
    return int.from_bytes(bus.read_i2c_block_data(
        addr, reg, 2), byteorder="big", signed=signed)


def write_int(bus, addr, reg, val):
    bus.write_i2c_block_data(addr, reg,
                             list(int.to_bytes(val, 2, "big")))


def read_temp_celsius(i2c_device: int, addr: int) -> Decimal:
    # logic adapted from https://os.mbed.com/teams/MaximIntegrated/code/MAX31725_Accurate_Temperature_Sensor/
    with SMBus(i2c_device) as bus:
        t = read_int(bus, addr, MAX31725_REG_TEMP, signed=True)
        temp = Decimal(t) * MAX31725_CF_LSB
    # todo: add 64 deg if extended format enabled
    return temp


def format_temp(temp: Decimal) -> str:
    return "{:.2f} °C".format(temp)


def format_addr(addr: int) -> str:
    return "0x{:2x}".format(addr)


def render_sensor(sensor, callback=None):
    with ui.card():
        ui.label("{} ({})".format(sensor.label, sensor.id)) \
            .tooltip(format_addr(sensor.addr))
        temp_label = ui.label("n/a")
        if callback is not None:
            ui.timer(1.0, lambda: callback(temp_label))


def main():
    ui.colors(primary='#6e93d6')
    ui.markdown('## Hestia dashboard')

    ui.markdown('### Primary sensors')
    with ui.row():
        for sensor in PRIMARY_SENSORS:
            render_sensor(sensor)

    ui.markdown('### Secondary sensors')
    with ui.row():
        for sensor in SECONDARY_SENSORS:
            render_sensor(sensor)

    ui.markdown('### Tertiary sensors')
    with ui.row():
        for sensor in TERTIARY_SENSORS:
            render_sensor(sensor, lambda label: label.set_text(
                format_temp(
                    read_temp_celsius(BEAGLE_I2C_BUS, sensor.addr))))

    ui.run(title='Hestia dashboard', port=8081)


# no guard clause, otherwise hot-reload doesn't work
main()
