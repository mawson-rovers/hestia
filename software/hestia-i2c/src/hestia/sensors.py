"""
This is an internal module. See :meth:`hestia.board.Hestia` for the public API.
"""

import logging
import math
from dataclasses import dataclass
from enum import Enum

from hestia.i2c import i2c_read_int

logger = logging.getLogger(name='hestia.sensors')
logger.setLevel(logging.DEBUG)

MSP430_I2C_ADDR = 0x08
MSP430_ADC_RESOLUTION = 1 << 12

NB21K00103_THERMISTOR_B_VALUE = 3630
ZERO_CELSIUS_IN_KELVIN = 273.15
NB21K00103_THERMISTOR_REF_TEMP_K = 25.0 + ZERO_CELSIUS_IN_KELVIN

ADS7828_I2C_ADDR = 0x4a
ADS7828_ADC_RESOLUTION = 1 << 12

MAX31725_REG_TEMP = 0x00
MAX31725_REG_CONFIG = 0x01
MAX31725_REG_THYST_LOW_TRIP = 0x02
MAX31725_REG_TOS_HIGH_TRIP = 0x03
MAX31725_REG_MAX = 0x03
MAX31725_CF_LSB = 0.00390625


class SensorInterface(str, Enum):
    MSP430 = 'MSP430'
    ADS7828 = 'ADS7828'
    MAX31725 = 'MAX31725'


@dataclass(frozen=True)
class Sensor:
    id: str
    iface: SensorInterface
    addr: int
    label: str
    pos_x: float = 0.0
    pos_y: float = 0.0

    def read_temp(self):
        if self.iface == SensorInterface.MSP430:
            return read_msp430_temp(self.addr)
        elif self.iface == SensorInterface.ADS7828:
            return read_ads7828_temp(self.addr)
        elif self.iface == SensorInterface.MAX31725:
            return read_max31725_temp(self.addr)
        else:
            logger.warning('Unknown sensor interface: %s', self.iface)
            return math.nan


def read_max31725_temp(addr: int) -> float:
    # logic adapted from https://os.mbed.com/teams/MaximIntegrated/code/MAX31725_Accurate_Temperature_Sensor/
    try:
        t = i2c_read_int(addr, MAX31725_REG_TEMP, signed=True)
        # todo: add 64 deg if extended format enabled
        return float(t) * MAX31725_CF_LSB
    except OSError as error:
        logger.warning("Could not read MAX31725 sensor 0x%02x: %s", addr, error)
        return math.nan


def read_msp430_temp(addr: int) -> float:
    try:
        adc_val = i2c_read_int(MSP430_I2C_ADDR, addr, byteorder="little", signed=False)
        logger.debug('Read value <%d> from ADC addr 0x%02x', adc_val, addr)
        return adc_val_to_temp(adc_val, MSP430_ADC_RESOLUTION)
    except OSError as error:
        logger.warning("Could not read MSP430 input 0x%02x: %s", addr, error)
        return math.nan


def adc_val_to_temp(adc_val: int, adc_resolution: int) -> float:
    try:
        return (1 / (1 / NB21K00103_THERMISTOR_REF_TEMP_K +
                     1 / NB21K00103_THERMISTOR_B_VALUE *
                     math.log(adc_resolution / adc_val - 1)) - ZERO_CELSIUS_IN_KELVIN)
    except (ValueError, ZeroDivisionError):
        # return NaN if value out of range (zero/negative)
        return math.nan


def ads7828_channel_select(addr: int) -> int:
    # implement crazy channel select - top bit is odd/even, low bits are floor(addr/2)
    # see ADS7828 datasheet for more details
    return ((addr & 0x01) << 2) | (addr >> 1)


def ads7828_command(addr: int) -> int:
    # set SD = 1, PD0 = 1 (see ADS7828 datasheet, p11)
    return 0x84 | (ads7828_channel_select(addr) << 4)


def read_ads7828_temp(addr: int) -> float:
    try:
        adc_cmd = ads7828_command(addr)
        logger.debug('Converted addr 0x%02x to ADS7828 command: %s', addr, '{0:b}'.format(adc_cmd))
        adc_val = i2c_read_int(0x48, adc_cmd, byteorder="big", signed=False)
        logger.info('Read value <%d> from ADC addr 0x%02x', adc_val, addr)
        return adc_val_to_temp(adc_val, ADS7828_ADC_RESOLUTION)
    except OSError as error:
        logger.warning("Could not read ADS7828 input 0x%02x: %s", addr, error)
        return math.nan
