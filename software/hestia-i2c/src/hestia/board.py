import contextlib
import logging
import math
from dataclasses import dataclass
from enum import Enum
from time import sleep
from typing import Dict, List

from hestia import sensors, heater

logger = logging.getLogger('hestia.board')


class SensorInterface(str, Enum):
    MSP430 = 'MSP430'
    ADS7828 = 'ADS7828'
    MAX31725 = 'MAX31725'


@dataclass(frozen=True)
class Sensor:
    id: str
    iface: SensorInterface
    addr: int
    label: str
    pos_x: float = 0.0
    pos_y: float = 0.0


_sensors = [
    Sensor("TH1", SensorInterface.MSP430, 0x01, "Centre"),
    Sensor("TH2", SensorInterface.MSP430, 0x02, "Top-left of heater"),
    Sensor("TH3", SensorInterface.MSP430, 0x03, "Bottom-right of heater"),
    Sensor("J7", SensorInterface.MSP430, 0x04, "Mounted"),
    Sensor("J8", SensorInterface.MSP430, 0x05, "Mounted"),
    Sensor("J9", SensorInterface.MSP430, 0x06, "Mounted"),
    Sensor("J10", SensorInterface.MSP430, 0x07, "Mounted"),
    Sensor("J11", SensorInterface.MSP430, 0x08, "Mounted"),

    Sensor("TH4", SensorInterface.ADS7828, 0x00, "Centre"),
    Sensor("TH5", SensorInterface.ADS7828, 0x01, "Top-right"),
    Sensor("TH6", SensorInterface.ADS7828, 0x02, "Bottom-left of heater"),
    Sensor("J12", SensorInterface.ADS7828, 0x03, "Mounted"),
    Sensor("J13", SensorInterface.ADS7828, 0x04, "Mounted"),
    Sensor("J14", SensorInterface.ADS7828, 0x05, "Mounted"),
    Sensor("J15", SensorInterface.ADS7828, 0x06, "Mounted"),
    Sensor("J16", SensorInterface.ADS7828, 0x07, "Mounted"),

    Sensor("U4", SensorInterface.MAX31725, 0x48, "Top-left"),
    Sensor("U5", SensorInterface.MAX31725, 0x4F, "Top-right"),
    Sensor("U6", SensorInterface.MAX31725, 0x49, "Bottom-right"),
    Sensor("U7", SensorInterface.MAX31725, 0x4B, "Centre"),
]


class Hestia:
    def sensors(self) -> List[Sensor]:
        return _sensors.copy()

    def read_center_temp(self) -> float:
        assert _sensors[0].iface == SensorInterface.MSP430
        return sensors.read_msp430_temp(_sensors[0].addr)

    def read_sensor_values(self) -> Dict[Sensor, float]:
        values = {}
        for sensor in self.sensors():
            if sensor.iface == SensorInterface.MSP430:
                values[sensor] = sensors.read_msp430_temp(sensor.addr)
            elif sensor.iface == SensorInterface.ADS7828:
                values[sensor] = math.nan  # todo - implement ADS reads
            elif sensor.iface == SensorInterface.MAX31725:
                values[sensor] = sensors.read_max31725_temp(sensor.addr)
            else:
                logger.warning('Unknown sensor interface: %s', sensor.iface)
        return values

    @contextlib.contextmanager
    def heating(self, power_level: int = 50):
        heater.set_heater_pwm(power_level)
        heater.enable_heater()
        try:
            yield self
        finally:
            heater.disable_heater()

    def heating_thermostat(self, temp: int = 80):
        heater.set_heater_pwm(255)
        try:
            while True:
                t = self.read_center_temp()
                if t < temp - 1:
                    heater.enable_heater()
                else:
                    heater.disable_heater()
                sleep(1)
        finally:
            heater.disable_heater()  # always disable heater at end
