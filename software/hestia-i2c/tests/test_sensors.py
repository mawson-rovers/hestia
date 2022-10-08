import logging
import math

from hestia import Hestia

log = logging.getLogger(__name__)


def test_sensors():
    hestia = Hestia()
    for sensor, temp in hestia.read_sensor_values().items():
        log.info('%s (0x%02x) => %.2f', sensor.label, sensor.addr, temp)
        assert 10.0 <= temp <= 80.0 or math.isnan(temp)
