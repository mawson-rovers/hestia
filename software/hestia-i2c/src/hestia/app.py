import csv
import math
import os
from collections import deque
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any

from flask import Flask, jsonify, Response, render_template

from hestia import Hestia, StubHestia, logger, i2c

app = Flask("hestia")


@app.route("/")
def home():
    return render_template('home.html', log_files=get_log_files())


@app.route("/api")
def api():
    board = get_board()
    heater_enabled = board.is_heater_enabled()
    return jsonify({
        "api_urls": {
            "data": "/api/data",
            "log_data": "/api/log_data",
            "log_download": "/api/log/<filename>"
        },
        "sensors": {sensor.id: sensor for sensor in board.sensors},
        "center_temp": board.read_center_temp(),
        "heater_enabled": heater_enabled,
        "heater_pwm_freq": board.get_heater_pwm() if heater_enabled else None,
        "heater_voltage": 0.0,
        "heater_current": 0.0,
        "log_files": get_log_files(attrs=('name', 'url')),
    })


def get_log_files(attrs=('name', 'url', 'file')):
    def to_dict(f):
        return {'name': f.name, 'url': '/log/%s' % f.name, 'file': f}

    return [{k: v for k, v in to_dict(f).items() if k in attrs}
            for f in logger.get_log_files()]


def get_board():
    # switch to stub implementation for local dev
    return Hestia() if i2c.device_exists() else StubHestia()


@app.route("/api/data")
def data():
    board = get_board()
    timestamp = datetime.now().strftime("%Y-%m-%d %T.%f")
    return {sensor.id: [[timestamp, value]] if not math.isnan(value) else []
            for sensor, value in board.read_sensor_values().items()}


@app.route("/api/log_data")
def full_data():
    board = get_board()
    sensors = board.sensors
    log_data = get_data_from_logs()
    return jsonify({s.id: list([ld['timestamp'], float(ld[s.id])]
                               for ld in log_data
                               if ld[s.id] != "")
                    for s in sensors})


def get_data_from_logs() -> List[Dict[str, Any]]:
    try:
        file = logger.get_log_files()[0]
    except IndexError:
        return []
    with file.open('r') as fh:
        csv_reader = csv.DictReader(fh)
        return list(deque(csv_reader, maxlen=100))


@app.route("/log/<filename>")
def log(filename: str):
    try:
        file: Path = next(f for f in logger.get_log_files() if f.name == filename)
    except StopIteration:
        return jsonify({})

    def generate():
        with file.open('r') as fh:
            for line in fh:
                yield line

    return Response(generate(), mimetype='text/csv')


if __name__ == "__main__":
    app.run(debug=True, host='0.0.0.0', port=os.environ.get('FLASK_PORT', 5000))
