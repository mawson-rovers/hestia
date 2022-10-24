import logging

from hestia import sensors

log = logging.getLogger(__name__)


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
