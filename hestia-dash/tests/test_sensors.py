import logging
import math

from pytest import approx

from hestia import Hestia, sensors

log = logging.getLogger(__name__)

hestia = Hestia()


def test_sensors():
    for sensor, temp in hestia.read_sensor_values().items():
        log.info('%s (0x%02x, %s) => %.2f', sensor.id, sensor.addr, sensor.label, temp)
        assert 10.0 <= temp <= 80.0 or math.isnan(temp)


def test_ads7828_channel_select():
    assert sensors.ads7828_channel_select(0) == 0b000
    assert sensors.ads7828_channel_select(2) == 0b001
    assert sensors.ads7828_channel_select(4) == 0b010
    assert sensors.ads7828_channel_select(6) == 0b011
    assert sensors.ads7828_channel_select(1) == 0b100
    assert sensors.ads7828_channel_select(3) == 0b101
    assert sensors.ads7828_channel_select(5) == 0b110
    assert sensors.ads7828_channel_select(7) == 0b111


def test_ads7828_command():
    assert sensors.ads7828_command(0) == 0b10000100
    assert sensors.ads7828_command(1) == 0b11000100
    assert sensors.ads7828_command(2) == 0b10010100
    assert sensors.ads7828_command(3) == 0b11010100
    assert sensors.ads7828_command(4) == 0b10100100
    assert sensors.ads7828_command(5) == 0b11100100
    assert sensors.ads7828_command(6) == 0b10110100
    assert sensors.ads7828_command(7) == 0b11110100


def test_adc_val_to_temp():
    resolution = 4096
    assert sensors.adc_val_to_temp(1024, resolution) == approx(0.323, 0.001)
    assert sensors.adc_val_to_temp(2048, resolution) == approx(25.0, 0.001)
    assert sensors.adc_val_to_temp(3072, resolution) == approx(54.57, 0.001)
    assert math.isnan(sensors.adc_val_to_temp(resolution, resolution))
    assert math.isnan(sensors.adc_val_to_temp(0, resolution))
    assert math.isnan(sensors.adc_val_to_temp(-250, resolution))


def test_temp_to_adc_val():
    assert sensors.temp_to_adc_value(0) == 1012
    assert sensors.temp_to_adc_value(25) == 2048
    assert sensors.temp_to_adc_value(40) == 2629
    assert sensors.temp_to_adc_value(50) == 2947
    assert sensors.temp_to_adc_value(60) == 3204
    assert sensors.temp_to_adc_value(70) == 3406
    assert sensors.temp_to_adc_value(80) == 3561
