import logging
import math

from hestia.i2c import i2c_read_int

logger = logging.getLogger(name='hestia.sensors')
logger.setLevel(logging.DEBUG)

MSP430_I2C_ADDR = 0x08

MSP430_ADC_RESOLUTION = 1 << 12
NB21K00103_THERMISTOR_B_VALUE = 3630
ZERO_CELSIUS_IN_KELVIN = 273.15
NB21K00103_THERMISTOR_REF_TEMP_K = 25.0 + ZERO_CELSIUS_IN_KELVIN

ADS7828_I2C_ADDR = 0x4A

MAX31725_REG_TEMP = 0x00
MAX31725_REG_CONFIG = 0x01
MAX31725_REG_THYST_LOW_TRIP = 0x02
MAX31725_REG_TOS_HIGH_TRIP = 0x03
MAX31725_REG_MAX = 0x03
MAX31725_CF_LSB = 0.00390625


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
        return (1 / (1 / NB21K00103_THERMISTOR_REF_TEMP_K +
                     1 / NB21K00103_THERMISTOR_B_VALUE *
                     math.log(MSP430_ADC_RESOLUTION / adc_val - 1)) - ZERO_CELSIUS_IN_KELVIN)
    except (ValueError, ZeroDivisionError):
        # ignore values out of range (zero/negative)
        return math.nan
    except OSError as error:
        logger.warning("Could not read MSP430 input 0x%02x: %s", addr, error)
        return math.nan
