## Hestia dashboard

This is a dashboard for Hestia built with [NiceGUI](https://github.com/zauberzeug/nicegui).

It requires the Hestia board to be connected via I2C, and has been tested with Python 3.8.

To get started, create a virtual env, install dependencies then run `dashboard.py`:

```shell
$ python3 -m venv venv             # create virtual env
$ source venv/bin/activate         # activate venv
$ pip install -r requirements.txt  # install requirements
$ ./dashboard.py                   # run the dashboard
NiceGUI ready to go on http://0.0.0.0:8081
```

By default, the web UI is available on port 8081.
