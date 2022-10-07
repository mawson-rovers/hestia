import logging
from time import sleep

from hestia import sensors, heater


log = logging.getLogger(__name__)


def test_heater_works():
    sensor = sensors.PRIMARY_SENSORS[0]  # TH1 centre thermistor
    start_temp = sensors.read_msp430_temp(sensor.addr)
    log.info('Start temp: %s (0x%02x) => %.2f', sensor.label, sensor.addr, start_temp)
    heater.enable_heater()
    sleep(30)
    heater.disable_heater()
    finish_temp = sensors.read_msp430_temp(sensor.addr)
    log.info('Finish temp: %s (0x%02x) => %.2f', sensor.label, sensor.addr, finish_temp)
    assert 5.0 < (finish_temp - start_temp) <= 40.0, "Should increase temperature a bit"
