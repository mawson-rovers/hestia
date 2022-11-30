"""
This is an internal module. See :meth:`hestia.board.Hestia` for the public API.
"""

import logging
from enum import Enum

from hestia.i2c import i2c_write_int, i2c_read_int

logger = logging.getLogger(name='hestia.heater')
logger.setLevel(logging.DEBUG)

MSP430_I2C_ADDR = 0x08
MSP430_READ_HEATER_MODE = 0x20
MSP430_READ_HEATER_PWM_FREQ = 0x23
MSP430_WRITE_HEATER_MODE = 0x40
MSP430_WRITE_PWM_FREQUENCY = 0x43


class HeaterMode(Enum):
    OFF = 0x0
    PID = 0x1  # temperature controlled
    PWM = 0x2  # fixed power input
    UNKNOWN = 0xFF


def enable_heater():
    logger.info('Enabling heater')
    i2c_write_int(MSP430_I2C_ADDR, MSP430_WRITE_HEATER_MODE, HeaterMode.PWM.value, byteorder="little")


def disable_heater():
    logger.info('Disabling heater')
    i2c_write_int(MSP430_I2C_ADDR, MSP430_WRITE_HEATER_MODE, HeaterMode.OFF.value, byteorder="little")


def set_heater_pwm(pwm_freq: int):
    logger.info('Setting heater power level %d' % pwm_freq)
    i2c_write_int(MSP430_I2C_ADDR, MSP430_WRITE_PWM_FREQUENCY, pwm_freq, byteorder="little")


def is_enabled() -> bool:
    logger.info('Reading heater mode')
    mode = i2c_read_int(MSP430_I2C_ADDR, MSP430_READ_HEATER_MODE, byteorder="little")
    logger.info('Read heater mode: %d' % mode)
    return HeaterMode(mode) != HeaterMode.OFF  # throws ValueError if unknown value


def get_heater_pwm() -> int:
    logger.info('Reading heater power level')
    return i2c_read_int(MSP430_I2C_ADDR, MSP430_READ_HEATER_PWM_FREQ, byteorder="little")

