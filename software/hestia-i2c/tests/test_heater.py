import logging
from time import sleep

from hestia import sensors, heater, Hestia

log = logging.getLogger(__name__)

hestia = Hestia()


def test_heater_works():
    start_temp = hestia.read_center_temp()
    log.info('Start temp: %.2f', start_temp)
    heater.set_heater_pwm(50)  # default
    with heater.on():
        sleep(30)
    finish_temp = hestia.read_center_temp()
    log.info('Finish temp: %.2f', finish_temp)
    assert 5.0 < (finish_temp - start_temp) <= 40.0, "Should increase temperature a bit"


def test_heater_full_power():
    start_temp = hestia.read_center_temp()
    log.info('Start temp: %.2f', start_temp)
    heater.set_heater_pwm(255)
    with heater.on():
        sleep(30)
    finish_temp = hestia.read_center_temp()
    log.info('Finish temp: %.2f', finish_temp)
    assert 5.0 < (finish_temp - start_temp) <= 40.0, "Should increase temperature a bit"
