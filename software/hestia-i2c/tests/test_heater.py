import logging
import math
from time import sleep

from hestia import Hestia

log = logging.getLogger(__name__)

hestia = Hestia()


def test_sensors():
    for sensor, temp in hestia.read_sensor_values().items():
        log.info('%s (0x%02x) => %.2f', sensor.label, sensor.addr, temp)
        assert 10.0 <= temp <= 80.0 or math.isnan(temp)


def test_heater_works():
    start_temp = hestia.read_center_temp()
    log.info('Start temp: %.2f', start_temp)
    with hestia.heating():
        sleep(30)
    finish_temp = hestia.read_center_temp()
    log.info('Finish temp: %.2f', finish_temp)
    assert 5.0 < (finish_temp - start_temp) <= 40.0, "Should increase temperature a bit"


def test_heater_full_power():
    start_temp = hestia.read_center_temp()
    log.info('Start temp: %.2f', start_temp)
    with hestia.heating(255):
        sleep(10)
    finish_temp = hestia.read_center_temp()
    log.info('Finish temp: %.2f', finish_temp)
    assert 5.0 < (finish_temp - start_temp) <= 40.0, "Should increase temperature a bit"
