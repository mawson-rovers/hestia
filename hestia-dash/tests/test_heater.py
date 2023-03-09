import logging
from time import sleep

from hestia import Hestia

log = logging.getLogger(__name__)

hestia = Hestia()


def test_heater_works():
    start_temp = hestia.read_center_temp()
    log.info('Start temp: %.2f', start_temp)
    assert not hestia.is_heater_enabled(), "Heater should not be enabled before"
    with hestia.heating():
        assert hestia.is_heater_enabled(), "Heater should be enabled"
        sleep(10)
    finish_temp = hestia.read_center_temp()
    assert not hestia.is_heater_enabled(), "Heater should not be enabled after"
    log.info('Finish temp: %.2f', finish_temp)
    assert 3.0 < (finish_temp - start_temp) <= 20.0, "Should increase temperature a little"


def test_heater_full_power():
    start_temp = hestia.read_center_temp()
    log.info('Start temp: %.2f', start_temp)
    with hestia.heating(255):
        sleep(10)
    finish_temp = hestia.read_center_temp()
    log.info('Finish temp: %.2f', finish_temp)
    assert 5.0 < (finish_temp - start_temp) <= 40.0, "Should increase temperature some more"
