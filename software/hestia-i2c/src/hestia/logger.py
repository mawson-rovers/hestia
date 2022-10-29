import math
import os
import sys
from datetime import datetime
from pathlib import Path
from time import sleep
from typing import List

from hestia import Hestia

LOG_PATH_ENV_VAR = 'HESTIA_LOG_PATH'


def log_path_configured() -> bool:
    return LOG_PATH_ENV_VAR in os.environ


def get_or_create_log_path() -> Path:
    if not log_path_configured():
        print("Specify HESTIA_LOG_PATH environment variable", file=sys.stderr)
        exit(1)

    log_path = Path(os.environ[LOG_PATH_ENV_VAR])
    if not log_path.exists():
        log_path.mkdir(parents=True)

    return log_path


def get_log_files() -> List[Path]:
    """
    :return: list of log files in reverse name order (should be most-recent first)
    """
    if not log_path_configured():
        return []

    log_path = Path(os.environ[LOG_PATH_ENV_VAR])
    if not log_path.exists():
        return []

    return sorted(list(log_path.glob('hestia-data-*.csv')),
                  key=lambda f: f.name,
                  reverse=True)


def main():
    log_path = get_or_create_log_path()

    start_date = datetime.now()
    file = log_path / ("hestia-data-%s.csv" % start_date.strftime('%Y-%m-%d'))
    write_header = not file.exists()

    hestia = Hestia()
    sensors = hestia.sensors()

    with file.open(mode='a', newline='\r\n') as f:
        print("Logging sensor data to %s..." % file, file=sys.stderr)

        if write_header:
            print('"timestamp"',
                  *['"%s"' % s.id for s in sensors],
                  file=f,
                  sep=",",
                  flush=True)

        while True:
            timestamp = datetime.now().strftime("%Y-%m-%d %T.%f")
            values = hestia.read_sensor_values()
            if not all(map(math.isnan, values.values())):
                print(timestamp,
                      *['%.4f' % values[s] if not math.isnan(values[s]) else '' for s in sensors],
                      file=f,
                      sep=",",
                      flush=True)
            sleep(5)
            if datetime.now().day > start_date.day:
                return  # start a new file if day ticks over


if __name__ == '__main__':
    while True:  # loops for each new day
        main()
