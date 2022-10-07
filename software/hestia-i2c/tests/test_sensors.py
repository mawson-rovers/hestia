import logging
import math

from hestia import sensors

BEAGLE_I2C_BUS = 2

log = logging.getLogger(__name__)


def test_max31725_sensors():
    for sensor in sensors.TERTIARY_SENSORS:
        temp = sensors.read_max31725_temp(BEAGLE_I2C_BUS, sensor.addr)
        log.info('%s (0x%02x) => %.2f', sensor.label, sensor.addr, temp)
        assert 10.0 <= temp <= 40.0


def test_msp430_sensors():
    for sensor in sensors.PRIMARY_SENSORS:
        temp = sensors.read_msp430_temp(BEAGLE_I2C_BUS, sensor.addr)
        log.info('%s (0x%02x) => %.2f', sensor.label, sensor.addr, temp)
        assert 10.0 <= temp <= 40.0 or math.isnan(temp)


def test_adc_sensors():
    # todo - implement read logic for discrete ADC
    pass

