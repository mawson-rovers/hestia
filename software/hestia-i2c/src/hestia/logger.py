import math
import os
import sys
from datetime import datetime
from pathlib import Path
from time import sleep

from hestia import Hestia


def main():
    try:
        log_path = Path(os.environ['HESTIA_LOG_PATH'])
    except KeyError:
        print("Specify HESTIA_LOG_PATH environment variable", file=sys.stderr)
        exit(1)

    if not log_path.exists():
        log_path.mkdir(parents=True)

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
