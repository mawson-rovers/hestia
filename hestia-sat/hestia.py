#!/usr/bin/env python3
import logging
import math
import sys
import unittest
from datetime import datetime
from enum import Enum
from time import sleep

from smbus2 import SMBus

logger = logging.getLogger(name='hestia.test')
logger.setLevel(logging.INFO)

BEAGLE_I2C_BUS = 2

MSP430_I2C_ADDR = 0x08
MSP430_ADC_RESOLUTION = 1 << 12
MSP430_READ_HEATER_MODE = 0x20
MSP430_READ_HEATER_PWM_FREQ = 0x23
MSP430_WRITE_HEATER_MODE = 0x40
MSP430_WRITE_PWM_FREQUENCY = 0x43

ADC_MIN_VALUE = 0x10  # disconnected ADC input fluctuates in low values close to zero

NB21K00103_THERMISTOR_B_VALUE = 3630
ZERO_CELSIUS_IN_KELVIN = 273.15
NB21K00103_THERMISTOR_REF_TEMP_K = 25.0 + ZERO_CELSIUS_IN_KELVIN

ADS7828_I2C_ADDR = 0x48  # switch to 0x4a for board v2
ADS7828_ADC_RESOLUTION = 1 << 12

MAX31725_REG_TEMP = 0x00
MAX31725_CF_LSB = 0.00390625

SENSORS = {
    "U5": lambda: read_max31725_temp(0x4F),
    "U6": lambda: read_max31725_temp(0x49),
    "U7": lambda: read_max31725_temp(0x4B),
    "TH1": lambda: read_msp430_temp(0x01),
    "TH2": lambda: read_msp430_temp(0x02),
}


def i2c_read_int(addr, reg, byteorder="big", signed=False):
    with SMBus(BEAGLE_I2C_BUS) as bus:
        return int.from_bytes(bus.read_i2c_block_data(
            addr, reg, 2), byteorder=byteorder, signed=signed)


def i2c_write_int(addr, reg, val, byteorder="big", signed=False):
    with SMBus(BEAGLE_I2C_BUS) as bus:
        bus.write_i2c_block_data(addr, reg, list(
            int.to_bytes(val, 2, byteorder=byteorder, signed=signed)))


def read_max31725_temp(addr: int) -> float:
    # logic adapted from https://os.mbed.com/teams/MaximIntegrated/code/MAX31725_Accurate_Temperature_Sensor/
    try:
        t = i2c_read_int(addr, MAX31725_REG_TEMP, signed=True)
        # todo: add 64 deg if extended format enabled
        logger.debug('Read value <%d> from MAX31725, addr 0x%02x', t, addr)
        return float(t) * MAX31725_CF_LSB
    except OSError as error:
        logger.warning("Could not read MAX31725 sensor 0x%02x: %s", addr, error)
        return math.nan


def read_msp430_temp(addr: int) -> float:
    try:
        adc_val = i2c_read_int(MSP430_I2C_ADDR, addr, byteorder="little", signed=False)
        logger.debug('Read value <%d> from MSP430, addr 0x%02x', adc_val, addr)
        return adc_val_to_temp(adc_val, MSP430_ADC_RESOLUTION)
    except OSError as error:
        logger.warning("Could not read MSP430 input 0x%02x: %s", addr, error)
        return math.nan


def adc_val_to_temp(adc_val: int, adc_resolution: int) -> float:
    if adc_val < ADC_MIN_VALUE:
        # return NaN if value too low (indicates no reading on ADC)
        return math.nan
    try:
        return (1 / (1 / NB21K00103_THERMISTOR_REF_TEMP_K +
                     1 / NB21K00103_THERMISTOR_B_VALUE *
                     math.log(adc_resolution / adc_val - 1)) - ZERO_CELSIUS_IN_KELVIN)
    except (ValueError, ZeroDivisionError):
        # return NaN if value out of range (zero/negative)
        return math.nan


def read_center_temp():
    return round(SENSORS["U7"](), 4)


class HeaterMode(Enum):
    OFF = 0x0
    PID = 0x1  # temperature controlled
    PWM = 0x2  # fixed power input
    UNKNOWN = 0xFF


def is_heater_enabled() -> bool:
    logger.debug('Reading heater mode')
    try:
        mode = i2c_read_int(MSP430_I2C_ADDR, MSP430_READ_HEATER_MODE, byteorder="little")
    except OSError as error:
        logger.warning("Could not read heater mode from MSP430: %s", error)
        return False
    logger.debug('Read heater mode: %d', mode)
    return HeaterMode(mode) != HeaterMode.OFF  # throws ValueError if unknown value


def enable_heater():
    logger.info('Enabling heater')
    i2c_write_int(MSP430_I2C_ADDR, MSP430_WRITE_HEATER_MODE, HeaterMode.PWM.value, byteorder="little")


def disable_heater():
    logger.info('Disabling heater')
    i2c_write_int(MSP430_I2C_ADDR, MSP430_WRITE_HEATER_MODE, HeaterMode.OFF.value, byteorder="little")


def get_heater_pwm() -> int:
    logger.debug('Reading heater power level')
    try:
        pwm_freq = i2c_read_int(MSP430_I2C_ADDR, MSP430_READ_HEATER_PWM_FREQ, byteorder="little")
    except OSError as error:
        logger.warning("Could not read heater power level from MSP430: %s", error)
        return -1
    logger.info('Read heater power level: %d', pwm_freq)
    return pwm_freq


def set_heater_pwm(pwm_freq: int):
    logger.info('Setting heater power level %d', pwm_freq)
    i2c_write_int(MSP430_I2C_ADDR, MSP430_WRITE_PWM_FREQUENCY, pwm_freq, byteorder="little")


class TestHestia(unittest.TestCase):
    def setUp(self) -> None:
        # ensure logging is printed to stdout
        if not logger.handlers:
            logger.addHandler(logging.StreamHandler(sys.stdout))

    def test_1_sensors(self):
        for sensor, read_fn in SENSORS.items():
            temp = round(read_fn(), 4)
            logger.info('%s => %.2f', sensor, temp)
            assert 10.0 <= temp <= 80.0 or math.isnan(temp)

    def test_2_heater_works(self):
        start_temp = read_center_temp()
        logger.info('Start temp: %.2f', start_temp)
        assert not is_heater_enabled(), "Heater should not be enabled before"
        try:
            set_heater_pwm(50)
            enable_heater()
            assert is_heater_enabled(), "Heater should be enabled"
            sleep(10)
        finally:
            disable_heater()
        finish_temp = SENSORS["U7"]()
        assert not is_heater_enabled(), "Heater should not be enabled after"
        logger.info('Finish temp: %.2f', finish_temp)
        assert 3.0 < (finish_temp - start_temp) <= 20.0, "Should increase temperature a little"

    def test_3_heater_full_power(self):
        start_temp = read_center_temp()
        logger.info('Start temp: %.2f', start_temp)
        try:
            set_heater_pwm(255)
            enable_heater()
            sleep(10)
        finally:
            disable_heater()
        finish_temp = read_center_temp()
        logger.info('Finish temp: %.2f', finish_temp)
        assert 5.0 < (finish_temp - start_temp) <= 40.0, "Should increase temperature some more"


def run_tests(args=()):
    unittest.main(argv=[sys.argv[0], *args])


def run_logger(f=sys.stdout):
    print('"timestamp"',
          *['"%s"' % s for s in SENSORS.keys()],
          '"heater"',
          file=f,
          sep=",",
          flush=True)
    while True:
        timestamp = datetime.now().strftime("%Y-%m-%d %T.%f")
        try:
            values = {s: read_fn() for s, read_fn in SENSORS.items()}
            heater_level = get_heater_pwm() if is_heater_enabled() else 0
            if not all(map(math.isnan, values.values())):
                print(timestamp,
                      *['%.4f' % values[s] if not math.isnan(values[s]) else '' for s in values.keys()],
                      heater_level,
                      file=f,
                      sep=",",
                      flush=True)
        except OSError as error:
            print("Failed to read board status: %s" % error, file=sys.stderr)
        sleep(5)


def run_heater(args):
    if not args:
        print("Heater status: " +
              ("ON" if is_heater_enabled() else "OFF"))
    else:
        if args[0] == "on":
            print("Enabling heater")
            enable_heater()
        else:
            print("Disabling heater")
            disable_heater()


def run_temps():
    for sensor, read_fn in SENSORS.items():
        print("%s => %.2f" % (sensor, read_fn()))


COMMANDS = {
    "log": lambda: run_logger(),
    "logger": lambda: run_logger(),
    "test": lambda args: run_tests(args),
    "temp": lambda: run_temps(),
    "heater": lambda args: run_heater(args),
    "power": lambda args: (set_heater_pwm(int(args[0])) if args
                           else print("Power level: %d" % get_heater_pwm())),
    "help": lambda: print("Available commands: %s" % list(COMMANDS.keys())),
}

if __name__ == '__main__':
    if len(sys.argv) > 1:
        cmd = COMMANDS[sys.argv[1]]
        cmd(sys.argv[2:]) if cmd.__code__.co_argcount else cmd()
    else:
        print("No command provided so running tests. Try 'help' for help.")
        run_tests()
