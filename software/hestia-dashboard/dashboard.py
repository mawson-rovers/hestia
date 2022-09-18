#!/usr/bin/env python3
import math
from dataclasses import dataclass
from decimal import Decimal
import logging
from random import random

from nicegui import ui
from smbus2 import SMBus

logger = logging.getLogger(name='hestia.dashboard')
logger.setLevel(logging.INFO)


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
    Sensor("J7", 0x04, "Mounted"),
    Sensor("J8", 0x05, "Mounted"),
    Sensor("J9", 0x06, "Mounted"),
    Sensor("J10", 0x07, "Mounted"),
    Sensor("J11", 0x08, "Mounted"),
]

MSP430_ADC_RESOLUTION = 1 << 12
NB21K00103_THERMISTOR_B_VALUE = 3630
ZERO_CELSIUS_IN_KELVIN = 273.15
NB21K00103_THERMISTOR_REF_TEMP_K = 25.0 + ZERO_CELSIUS_IN_KELVIN

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


def read_int(bus, addr, reg, byteorder="big", signed=False):
    return int.from_bytes(bus.read_i2c_block_data(
        addr, reg, 2), byteorder=byteorder, signed=signed)


def write_int(bus, addr, reg, val, byteorder="big", signed=False):
    bus.write_i2c_block_data(addr, reg, list(
        int.to_bytes(val, 2, byteorder=byteorder, signed=signed)))


def read_max31725_temp(i2c_device: int, addr: int) -> Decimal:
    # logic adapted from https://os.mbed.com/teams/MaximIntegrated/code/MAX31725_Accurate_Temperature_Sensor/
    try:
        with SMBus(i2c_device) as bus:
            t = read_int(bus, addr, MAX31725_REG_TEMP, signed=True)
            temp = Decimal(t) * MAX31725_CF_LSB
        # todo: add 64 deg if extended format enabled
        return temp
    except OSError as error:
        logger.warning("Could not read MAX31725 sensor 0x%02x: %s", addr, error)
        return Decimal(math.nan)


def read_msp430_temp(i2c_device: int, addr: int) -> Decimal:
    try:
        with SMBus(i2c_device) as bus:
            adc_val = read_int(bus, MSP430_I2C_ADDR, addr, byteorder="little", signed=False)
            logger.debug('Read value <%d> from ADC addr %s', adc_val, format_addr(addr))
            temp = (1 / (1 / NB21K00103_THERMISTOR_REF_TEMP_K +
                         1 / NB21K00103_THERMISTOR_B_VALUE *
                         math.log(MSP430_ADC_RESOLUTION / adc_val - 1)) - ZERO_CELSIUS_IN_KELVIN)
    except (ValueError, ZeroDivisionError):
        # ignore values out of range (zero/negative)
        return Decimal(math.nan)
    except OSError as error:
        logger.warning("Could not read MSP430 input 0x%02x: %s", addr, error)
        return Decimal(math.nan)
    return Decimal(temp)


def format_temp(temp: Decimal) -> str:
    return "{:.2f} °C".format(temp) if not temp.is_nan() else 'n/a'


def format_addr(addr: int) -> str:
    return "0x{:02x}".format(addr)


def render_sensor(sensor, callback=None):
    with ui.card():
        ui.label("{} ({})".format(sensor.label, sensor.id)) \
            .tooltip(format_addr(sensor.addr))
        temp_label = ui.label("n/a")
        if callback:
            ui.timer(1.0 + random(), lambda: callback(temp_label))


def main():
    ui.colors(primary='#6e93d6')
    ui.markdown('## Hestia dashboard')

    ui.markdown('### Primary sensors')
    with ui.row():
        for sensor in PRIMARY_SENSORS:
            render_sensor(sensor, lambda label, sensor2=sensor: label.set_text(format_temp(
                read_msp430_temp(BEAGLE_I2C_BUS, sensor2.addr))))

    ui.markdown('### Secondary sensors')
    with ui.row():
        for sensor in SECONDARY_SENSORS:
            render_sensor(sensor)

    ui.markdown('### Tertiary sensors')
    with ui.row():
        for sensor in TERTIARY_SENSORS:
            render_sensor(sensor, lambda label, sensor2=sensor: label.set_text(format_temp(
                read_max31725_temp(BEAGLE_I2C_BUS, sensor2.addr))))

    ui.run(title='Hestia dashboard', port=8081)


# no guard clause, otherwise hot-reload doesn't work
main()