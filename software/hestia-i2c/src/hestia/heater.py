"""
This is an internal module. See :meth:`hestia.board.Hestia` for the public API.
"""

import logging
from enum import Enum

from hestia.i2c import i2c_write_int

logger = logging.getLogger(name='hestia.heater')
logger.setLevel(logging.DEBUG)

MSP430_I2C_ADDR = 0x08
MSP430_REG_HEATER_MODE = 0x40
MSP430_REG_PWM_FREQUENCY = 0x43


class HeaterMode(Enum):
    OFF = 0x0
    PID = 0x1  # temperature controlled
    PWM = 0x2  # fixed power input
    UNKNOWN = 0xFF


def enable_heater():
    i2c_write_int(MSP430_I2C_ADDR, MSP430_REG_HEATER_MODE, HeaterMode.PWM.value, byteorder="little")


def disable_heater():
    i2c_write_int(MSP430_I2C_ADDR, MSP430_REG_HEATER_MODE, HeaterMode.OFF.value, byteorder="little")


def set_heater_pwm(pwm_freq: int):
    i2c_write_int(MSP430_I2C_ADDR, MSP430_REG_PWM_FREQUENCY, pwm_freq, byteorder="little")
