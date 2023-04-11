"""
This is an internal module. See :meth:`hestia.board.Hestia` for the public API.
"""

import logging
from enum import Enum

from hestia import sensors
from hestia.i2c import i2c_write_int, i2c_read_int

logger = logging.getLogger(name='hestia.heater')
# logger.setLevel(logging.DEBUG)

MSP430_I2C_ADDR = 0x08
MSP430_READ_HEATER_MODE = 0x20
MSP430_READ_HEATER_PWM_FREQ = 0x23
MSP430_WRITE_HEATER_MODE = 0x40
MSP430_WRITE_TARGET_TEMP = 0x41
MSP430_WRITE_TARGET_SENSOR = 0x42
MSP430_WRITE_PWM_FREQUENCY = 0x43


class HeaterMode(Enum):
    OFF = 0x0
    PID = 0x1  # temperature controlled
    PWM = 0x2  # fixed power input
    UNKNOWN = 0xFF


def set_heater_mode(mode: HeaterMode):
    logger.info("Setting heater mode to %s (%02x)", mode.name, mode.value)
    try:
        i2c_write_int(MSP430_I2C_ADDR, MSP430_WRITE_HEATER_MODE, mode.value, byteorder="little")
    except OSError as error:
        logger.warning("Could not set heater mode: %s", error)


def set_heater_power_level(pwm_freq: int):
    logger.info('Setting heater power level to %d', pwm_freq)
    i2c_write_int(MSP430_I2C_ADDR, MSP430_WRITE_PWM_FREQUENCY, pwm_freq, byteorder="little")


def set_heater_target_temp(temp: float):
    adc_value = sensors.temp_to_adc_value(temp)
    logger.info('Setting heater set point to %0.2f (ADC value: %d)', temp, adc_value)
    i2c_write_int(MSP430_I2C_ADDR, MSP430_WRITE_TARGET_TEMP, adc_value, byteorder="little")


def get_heater_mode() -> HeaterMode:
    logger.debug('Reading heater mode')
    try:
        mode = i2c_read_int(MSP430_I2C_ADDR, MSP430_READ_HEATER_MODE, byteorder="little")
        logger.info('Read heater mode: %d', mode)
        return HeaterMode(mode)
    except OSError as e:
        logger.warning("Could not read heater mode from MSP430: %s", e)
        return HeaterMode.UNKNOWN
    except ValueError:
        return HeaterMode.UNKNOWN


def get_heater_power_level() -> int:
    logger.debug('Reading heater power level')
    try:
        pwm_freq = i2c_read_int(MSP430_I2C_ADDR, MSP430_READ_HEATER_PWM_FREQ, byteorder="little")
    except OSError as error:
        logger.warning("Could not read heater power level from MSP430: %s", error)
        return -1
    logger.info('Read heater power level: %d', pwm_freq)
    return pwm_freq

