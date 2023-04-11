import csv
import logging
import math
import os
from collections import deque
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any

from flask import Flask, jsonify, Response, render_template, request, redirect, send_from_directory
from flask_compress import Compress

from hestia import Hestia, logger, i2c, stub_instance, stub_board, HeaterMode

app = Flask("hestia")
Compress(app)
app.config['JSON_SORT_KEYS'] = False
app.logger.setLevel(logging.INFO)


@app.get("/")
def home():
    return render_template('home.html', log_files=get_log_files())


@app.get("/api")
def api():
    return jsonify({
        "app_urls": {
            "home": "/",
            "api": "/api",
            "status": "/api/status",
            "data": "/api/data",
            "log_data": "/api/log_data",
            "log_download": "/log/<filename>"
        },
        "log_files": get_log_files(attrs=('name', 'url')),
    })


@app.get("/api/status")
def get_status():
    board = get_board()
    return jsonify({
        "center_temp": board.read_center_temp(),
        "heater_mode": board.get_heater_mode().name,
        "heater_duty": power_level_to_duty(board.get_heater_power_level()),
        "target_temp": board.get_target_temp(),
        "heater_voltage": 0.0,
        "heater_current": 0.0,
        "sensors": {sensor.id: sensor for sensor in board.sensors},
    })


@app.route("/api/status", methods=['POST'])
def post_status():
    board = get_board()
    data = request.json
    if 'heater_mode' in data:
        app.logger.info('Setting heater mode to %s', data['heater_mode'])
        board.set_heater_mode(HeaterMode[data['heater_mode']])
    if 'heater_duty' in data:
        power_level = duty_to_power_level(data['heater_duty'])
        app.logger.info('Setting heater power level to %d (%d%%)', power_level, data['heater_duty'])
        board.set_heater_pwm(power_level)
    if 'target_temp' in data:
        app.logger.info('Setting target temperature to %d°C', data['target_temp'])
        board.set_target_temp(data['target_temp'])
    return redirect('/api/status')


def duty_to_power_level(duty: int):
    return round(duty / 100 * 255)


def power_level_to_duty(power_level: int):
    return round(power_level / 255 * 100)


def get_log_files(attrs=('name', 'url', 'file')):
    def to_dict(f):
        return {'name': f.name, 'url': '/log/%s' % f.name, 'file': f}

    return [{k: v for k, v in to_dict(f).items() if k in attrs}
            for f in logger.get_log_files()]


def get_board():
    # switch to stub implementation for local dev
    return Hestia() if i2c.device_exists() else stub_instance


@app.get("/api/data")
def get_data():
    board = get_board()
    timestamp = datetime.now().strftime("%Y-%m-%d %T.%f")
    heater_level = board.get_heater_power_level() if board.get_heater_mode() != HeaterMode.OFF else 0
    sensor_readings = {sensor.id: [[timestamp, value]] if not math.isnan(value) else []
                       for sensor, value in board.read_sensor_values().items()}
    return {**sensor_readings, 'heater': [[timestamp, heater_level]]}


@app.get("/api/log_data")
def get_log_data():
    board = get_board()
    sensor_ids = map(lambda s: s.id, board.sensors)
    log_data = read_recent_logs()
    return jsonify({id: list([ld['timestamp'], float(ld[id])]
                             for ld in log_data
                             if id in ld and ld[id] is not None and ld[id] != "")
                    for id in (*sensor_ids, 'heater')})


def read_recent_logs() -> List[Dict[str, Any]]:
    if not i2c.device_exists():
        return stub_board.get_stub_logs()
    try:
        file = logger.get_log_files()[0]
    except IndexError:
        return []
    with file.open('r') as fh:
        csv_reader = csv.DictReader(fh)
        return list(deque(csv_reader, maxlen=1500))  # last 125 mins of logs


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


@app.route('/favicon.ico')
def favicon():
    return send_from_directory(os.path.join(app.root_path, 'static'),
                               'hestia-favicon.ico', mimetype='image/vnd.microsoft.icon')


if __name__ == "__main__":
    app.run(debug=True, host='0.0.0.0', port=os.environ.get('FLASK_PORT', 5000))
