import math
from datetime import datetime, timedelta
from random import random
from typing import Dict, Optional

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
        self.heater_started: Optional[datetime] = None

    def read_sensor_values(self) -> Dict[Sensor, float]:
        return {s: self.read_sensor(s.id) for s in self.sensors}

    def read_sensor(self, sensor_id):
        base_val = _stub_values.get(sensor_id, math.nan) + random() * 5 - 2
        if self.heater_enabled:
            heating_duration: timedelta = datetime.now() - self.heater_started
            max_temp: float = 50 + self.heater_pwm / 4
            return round(min(base_val + self.heater_pwm / 50 * heating_duration.total_seconds(),
                             max_temp), 4)
        else:
            return round(base_val, 4)

    def read_center_temp(self) -> float:
        return self.read_sensor('TH1')

    def is_heater_enabled(self) -> bool:
        return self.heater_enabled

    def get_heater_pwm(self) -> int:
        return self.heater_pwm

    def set_heater_pwm(self, power_level: int):
        self.heater_pwm = power_level

    def enable_heater(self):
        self.heater_enabled = True
        self.heater_started = datetime.now()

    def disable_heater(self):
        self.heater_enabled = False
