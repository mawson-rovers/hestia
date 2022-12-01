import math
from typing import Dict

from hestia import Hestia
from hestia.board import Sensor

_stub_values = {
    "TH1": 24.64,
    "TH2": 23.62,
    "TH3": 23.86,
    "J7": math.nan,
    "J8": math.nan,
    "J9": math.nan,
    "J10": math.nan,

    "TH4": math.nan,
    "TH5": math.nan,
    "TH6": math.nan,
    "J12": math.nan,
    "J13": math.nan,
    "J14": math.nan,
    "J15": math.nan,
    "J16": math.nan,

    "U4": 25.10,
    "U5": 24.87,
    "U6": 24.75,
    "U7": 26.32,
}


class StubHestia(Hestia):
    def __init__(self):
        super().__init__()
        self.heater_enabled: bool = False
        self.heater_pwm: int = 50

    def read_sensor_values(self) -> Dict[Sensor, float]:
        return {s: _stub_values.get(s.id, math.nan) for s in self.sensors}

    def read_center_temp(self) -> float:
        return _stub_values['TH1']

    def is_heater_enabled(self) -> bool:
        return self.heater_enabled

    def get_heater_pwm(self) -> int:
        return self.heater_pwm

    def set_heater_pwm(self, power_level: int):
        self.heater_pwm = power_level

    def enable_heater(self):
        self.heater_enabled = True

    def disable_heater(self):
        self.heater_enabled = False
