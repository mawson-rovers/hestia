import contextlib
import logging
from collections import OrderedDict
from time import sleep
from typing import Dict

from hestia import heater
from hestia.i2c import i2c_write_int
from hestia.sensors import Sensor, SensorInterface

logger = logging.getLogger('hestia.board')


MSP430_I2C_ADDR = 0x08
MSP430_COMMAND_RESET = 0x50

_sensors = [
    Sensor("TH1", SensorInterface.MSP430, 0x01, "Centre", -42.0135, 43.18),
    Sensor("TH2", SensorInterface.MSP430, 0x02, "Top-left of heater", -35.7124, 54.61),
    Sensor("TH3", SensorInterface.MSP430, 0x03, "Bottom-right of heater", -53.88, 33.496),

    Sensor("U4", SensorInterface.MAX31725, 0x48, "Top-left", -15.976, 75.225),
    Sensor("U5", SensorInterface.MAX31725, 0x4F, "Top-right", 81.788, 75.692),
    Sensor("U6", SensorInterface.MAX31725, 0x49, "Bottom-right", -82.296, 12.8535),
    Sensor("U7", SensorInterface.MAX31725, 0x4B, "Centre", 46.228, 47.752),

    Sensor("TH4", SensorInterface.ADS7828, 0x00, "Centre", -45.8705, 43.18),
    Sensor("TH5", SensorInterface.ADS7828, 0x01, "Top-right", -77.9814, 75.0769),
    Sensor("TH6", SensorInterface.ADS7828, 0x02, "Bottom-left of heater", 33.274, 30.226),

    Sensor("J7", SensorInterface.MSP430, 0x04, "Mounted"),
    Sensor("J8", SensorInterface.MSP430, 0x05, "Mounted"),
    Sensor("J9", SensorInterface.MSP430, 0x06, "Mounted"),
    Sensor("J10", SensorInterface.MSP430, 0x07, "Mounted"),
    Sensor("J11", SensorInterface.MSP430, 0x08, "Mounted"),

    Sensor("J12", SensorInterface.ADS7828, 0x03, "Mounted"),
    Sensor("J13", SensorInterface.ADS7828, 0x04, "Mounted"),
    Sensor("J14", SensorInterface.ADS7828, 0x05, "Mounted"),
    Sensor("J15", SensorInterface.ADS7828, 0x06, "Mounted"),
    Sensor("J16", SensorInterface.ADS7828, 0x07, "Mounted"),
]


# noinspection PyMethodMayBeStatic
class Hestia:
    def __init__(self) -> None:
        super().__init__()
        self.sensors = _sensors
        self.center_sensor = self.sensors[0]

    def read_center_temp(self) -> float:
        return self.center_sensor.read_temp()

    def read_sensor_values(self) -> Dict[Sensor, float]:
        return OrderedDict((s, s.read_temp()) for s in self.sensors)

    def is_heater_enabled(self) -> bool:
        return heater.is_enabled()

    def get_heater_pwm(self) -> int:
        return heater.get_heater_pwm()

    def set_heater_pwm(self, power_level: int):
        heater.set_heater_pwm(power_level)

    def enable_heater(self):
        heater.enable_heater()

    def disable_heater(self):
        heater.disable_heater()

    @contextlib.contextmanager
    def heating(self, power_level: int = 50):
        self.set_heater_pwm(power_level)
        self.enable_heater()
        try:
            yield self
        finally:
            self.disable_heater()

    def heating_thermostat(self, temp: int = 80):
        self.set_heater_pwm(255)
        try:
            while True:
                t = self.read_center_temp()
                if t < temp - 1:
                    self.enable_heater()
                else:
                    self.disable_heater()
                sleep(1)
        finally:
            self.disable_heater()  # always disable heater at end

    def reset(self):
        logger.info("Sending reset command")
        i2c_write_int(MSP430_I2C_ADDR, MSP430_COMMAND_RESET, 0, byteorder="little")
