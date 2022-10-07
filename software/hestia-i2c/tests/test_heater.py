import logging
from time import sleep

from hestia import sensors, heater

log = logging.getLogger(__name__)

sensor = sensors.PRIMARY_SENSORS[0]  # TH1 centre thermistor


def test_heater_works():
    start_temp = sensors.read_msp430_temp(sensor.addr)
    log.info('Start temp: %s (0x%02x) => %.2f', sensor.label, sensor.addr, start_temp)
    heater.set_heater_pwm(50)  # default
    with heater.on():
        sleep(30)
    finish_temp = sensors.read_msp430_temp(sensor.addr)
    log.info('Finish temp: %s (0x%02x) => %.2f', sensor.label, sensor.addr, finish_temp)
    assert 5.0 < (finish_temp - start_temp) <= 40.0, "Should increase temperature a bit"


def test_heater_full_power():
    start_temp = sensors.read_msp430_temp(sensor.addr)
    log.info('Start temp: %s (0x%02x) => %.2f', sensor.label, sensor.addr, start_temp)
    heater.set_heater_pwm(255)
    with heater.on():
        sleep(30)
    finish_temp = sensors.read_msp430_temp(sensor.addr)
    log.info('Finish temp: %s (0x%02x) => %.2f', sensor.label, sensor.addr, finish_temp)
    assert 5.0 < (finish_temp - start_temp) <= 40.0, "Should increase temperature a bit"
