import math

from flask import Flask, jsonify
from hestia import Hestia, StubHestia

app = Flask("hestia")


@app.route("/")
def home():
    hestia = Hestia()
    # hestia = StubHestia()  # use the stub implementation for local dev

    sensor_data = {sensor.id: {"sensor": sensor, "value": value}
                   for sensor, value in hestia.read_sensor_values().items()
                   if not math.isnan(value)}
    return jsonify({
        "sensors": sensor_data,
        "center_temp": hestia.read_center_temp(),
        "heater": "OFF",
    })
