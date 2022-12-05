import csv
import logging
import math
import os
from collections import deque
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any

from flask import Flask, jsonify, Response, render_template, request, redirect, send_from_directory

from hestia import Hestia, logger, i2c, stub_instance

app = Flask("hestia")
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
        "sensors": {sensor.id: sensor for sensor in board.sensors},
        "center_temp": board.read_center_temp(),
        "heater_enabled": board.is_heater_enabled(),
        "heater_pwm_freq": board.get_heater_pwm(),
        "heater_voltage": 0.0,
        "heater_current": 0.0,
    })


@app.route("/api/status", methods=['POST'])
def post_status():
    board = get_board()
    data = request.json
    if 'heater_enabled' in data:
        app.logger.info('Enabling heater' if data['heater_enabled'] else 'Disabling heater')
        board.enable_heater() if data['heater_enabled'] else board.disable_heater()
    if 'heater_pwm_freq' in data:
        app.logger.info('Setting heater power level to %d', data['heater_pwm_freq'])
        board.set_heater_pwm(data['heater_pwm_freq'])
    return redirect('/api/status')


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
    return {sensor.id: [[timestamp, value]] if not math.isnan(value) else []
            for sensor, value in board.read_sensor_values().items()}


@app.get("/api/log_data")
def get_log_data():
    board = get_board()
    sensors = board.sensors
    log_data = read_recent_logs()
    return jsonify({s.id: list([ld['timestamp'], float(ld[s.id])]
                               for ld in log_data
                               if ld[s.id] != "")
                    for s in sensors})


def read_recent_logs() -> List[Dict[str, Any]]:
    try:
        file = logger.get_log_files()[0]
    except IndexError:
        return []
    with file.open('r') as fh:
        csv_reader = csv.DictReader(fh)
        return list(deque(csv_reader, maxlen=500))  # last 40 mins of logs


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
