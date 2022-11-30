import logging
from time import sleep

from hestia import Hestia

log = logging.getLogger(__name__)

hestia = Hestia()


def test_reset():
    assert 10.0 <= hestia.read_center_temp() <= 80.0, "start temp"
    hestia.reset()
    # reading immediately sometimes fails, but can't test due to race condition
    sleep(2)  # wait for reset
    assert 10.0 <= hestia.read_center_temp() <= 80.0, "after temp"
