import math
import os
from pathlib import Path

from flask import Flask, jsonify, Response, stream_with_context, render_template
from hestia import Hestia, StubHestia, logger, i2c

app = Flask("hestia")


@app.route("/")
def home():
    return render_template('home.html', log_files=get_log_files())


@app.route("/api")
def api():
    board = get_board()

    sensor_data = {sensor.id: {"sensor": sensor, "value": value}
                   for sensor, value in board.read_sensor_values().items()
                   if not math.isnan(value)}
    return jsonify({
        "sensors": sensor_data,
        "center_temp": board.read_center_temp(),
        "heater": "OFF",
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

