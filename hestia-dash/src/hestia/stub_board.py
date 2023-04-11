import math
import re
from datetime import datetime, timedelta
from random import random
from typing import Dict, Optional, List, Any

from hestia import Hestia, HeaterMode
from hestia.board import Sensor

_stub_values = {
    "TH1": 24.64,
    "TH2": 23.62,
    "TH3": 23.86,
    "J7": 24.35,
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


def get_stub_logs() -> List[Dict[str, Any]]:
    data = []
    end_date = datetime.now()
    d = end_date - timedelta(minutes=120)
    while d <= end_date:
        entry = {
            "timestamp": d.strftime("%Y-%m-%d %T.%f"),
            "heater": int(d.timestamp()) % 256,
        }
        ts = d.timestamp()
        for sensor_id, value in _stub_values.items():
            if not math.isnan(value):
                int_id = int(re.sub('[A-Z]', '', sensor_id))
                ts_offset = ts / 80
                entry[sensor_id] = value + (math.sin(ts_offset * int_id) + math.cos(ts_offset * 7)) * 5
        data.append(entry)
        d += timedelta(seconds=5)
    return data


class StubHestia(Hestia):
    def __init__(self):
        super().__init__()
        self.heater_mode: HeaterMode = HeaterMode.OFF
        self.heater_pwm: int = 50
        self.heater_started: Optional[datetime] = None

    def read_sensor_values(self) -> Dict[Sensor, float]:
        return {s: self.read_sensor(s.id) for s in self.sensors}

    def read_sensor(self, sensor_id):
        base_val = _stub_values.get(sensor_id, math.nan) + random() * 5 - 2
        if self.heater_mode != HeaterMode.OFF:
            heating_duration: timedelta = datetime.now() - self.heater_started
            max_temp: float = 50 + self.heater_pwm / 4
            return round(min(base_val + self.heater_pwm / 50 * heating_duration.total_seconds(),
                             max_temp), 4)
        else:
            return round(base_val, 4)

    def read_center_temp(self) -> float:
        return self.read_sensor('TH1')

    def get_heater_mode(self) -> HeaterMode:
        return self.heater_mode

    def get_heater_power_level(self) -> int:
        return self.heater_pwm

    def set_heater_pwm(self, power_level: int):
        self.heater_pwm = power_level

    def set_heater_mode(self, mode: HeaterMode):
        self.heater_mode = mode
        if self.heater_mode != HeaterMode.OFF:
            self.heater_started = datetime.now()
