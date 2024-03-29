import logging
from time import sleep

from hestia import Hestia, HeaterMode

log = logging.getLogger(__name__)

hestia = Hestia()


def test_heater_works():
    start_temp = hestia.read_center_temp()
    log.info('Start temp: %.2f', start_temp)
    assert hestia.get_heater_mode() == HeaterMode.OFF, "Heater should not be enabled before"
    hestia.set_heater_pwm(50)
    hestia.set_heater_mode(HeaterMode.PWM)
    try:
        assert hestia.get_heater_mode() == HeaterMode.PWM, "Heater should be enabled"
        sleep(10)
    finally:
        hestia.set_heater_mode(HeaterMode.OFF)
    finish_temp = hestia.read_center_temp()
    assert hestia.get_heater_mode() == HeaterMode.OFF, "Heater should not be enabled after"
    log.info('Finish temp: %.2f', finish_temp)
    assert 3.0 < (finish_temp - start_temp) <= 20.0, "Should increase temperature a little"


def test_heater_full_power():
    start_temp = hestia.read_center_temp()
    log.info('Start temp: %.2f', start_temp)
    hestia.set_heater_pwm(255)
    hestia.set_heater_mode(HeaterMode.PWM)
    try:
        sleep(10)
    finally:
        hestia.set_heater_mode(HeaterMode.OFF)
    finish_temp = hestia.read_center_temp()
    log.info('Finish temp: %.2f', finish_temp)
    assert 5.0 < (finish_temp - start_temp) <= 40.0, "Should increase temperature some more"
