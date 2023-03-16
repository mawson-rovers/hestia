# hestia-dash

This is the dashboard webapp for Hestia, written in Python. It is designed to run on Python 3.7
and later, because this is the latest version available for the default Debian OS on the
BeagleBone Black.

### BeagleBone Black setup

The dashboard is designed to run on a BeagleBone Black (BBB) payload computer, connected to the Hestia board over I2C.

To set up a new BBB from scratch, you need to:

* Connect the BBB via USB to your computer, log in via SSH using `debian`/`temppwd`

* Configure internet sharing so the BBB can connect to the internet via the USB connection:
    * For Mac, enable Internet Sharing in the Sharing preference pane, then run `/opt/scripts/network/usb_mac_ics.sh` on
      the BBB
    * For Linux, you will need to configure NAT on the USB interface, then can run a similar
      script `/opt/scripts/network/`

* Install driver and configure the wi-fi with the TP-Link TL-WN725N dongle:

```sh
$ sudo apt update
$ sudo apt install git make gcc build-essential linux-headers-4.19.94-ti-r42
$ mkdir -p ~/src && cd ~/src
$ git clone https://github.com/lwfinger/rtl8188eu.git
$ cd rtl8188eu
$ make all
$ sudo make install
$ sudo reboot
$ sudo connmanctl
#connmanctl> tether wifi off
#connmanctl> enable wifi
#connmanctl> scan wifi
#connmanctl> services
#connmanctl> agent on
#connmanctl> connect wifi_*_managed_psk  # <- get this value from the services list above
#connmanctl> quit
```

(For the TP-Link AC600 Wi-Fi dongle - the one with the antenna - install the
[rtl8812au](https://github.com/aircrack-ng/rtl8812au.git) driver instead.)

* Install OS dependencies and check out the `hestia` source code from GitHub:

```sh
$ sudo apt install git python3-venv
$ mkdir -p ~/src/mawson && cd ~/src/mawson
$ git clone https://github.com/mawson-rovers/hestia
```

### Setting up the Python code and running tests

* Create a Python virtual environment and install the dependencies:

```sh
$ cd ~/src/mawson/hestia/hestia-dash
$ python3 -m venv venv
$ source venv/bin/activate
(venv) $ pip install -r requirements.txt
```

* You can now run the tests for the payload control software:

```sh
(venv) $ pytest
```

You should see some testing output and results, depending on whether the board is connected and all the
sensors are working.

If you want to disable some sensors during testing and select specific tests to run, you can do this with the
`HESTIA_SENSOR_DISABLE` environment variable and pytest's `-k` flag:

```sh
(venv) $ HESTIA_SENSOR_DISABLE=U4,TH5 pytest -k sensor
```

### Setting up the system services

The dashboard is designed to run as two system services when the BeagleBone is powered on, configured via two
systemd configuration files found in `config/` in the repository.

Start by reviewing the configuration files to ensure they match the system paths:

* `config/hestia-logger.service`
  * ensure the `WorkingDirectory` and `ExecStart` directories match where the code was checked out
  * ensure that `HESTIA_LOG_PATH` is where you want to store log files (by default it will store them under
    `/home/debian/data`)
* `config/hestia-app.service`
  * ensure the `WorkingDirectory` and `ExecStart` directories match where the code was checked out
  * ensure that `HESTIA_LOG_PATH` matches the configuration for hestia-logger
  * set `HESTIA_SENSOR_DISABLE` if you want to disable any sensors (as shown above)

Then you can link the files into the systemd configuration and load them up:

```sh
$ cd /etc/systemd/system
$ sudo ln -s /home/debian/src/mawson/hestia/hestia-dash/config/*.service .
$ sudo systemctl daemon-reload
$ sudo systemctl enable hestia-logger
$ sudo systemctl enable hestia-app
```

From now on, the services will start automatically at boot. The Python webapp runs in development mode, so it will also
automatically pick up any changes you make to the source code.

To restart the services, you can use `sudo systemctl restart hestia-logger` (or `hestia-app`), but any changes to the
service config files will require `sudo systemctl daemon-reload` first.

You can watch the log file output from the webapp using `journalctl`:

```sh
$ journalctl -f -u hestia-app
Mar 10 03:17:49 beaglebone systemd[1]: Started Hestia app.
Mar 10 03:17:52 beaglebone python[2483]:  * Serving Flask app 'hestia'
Mar 10 03:17:52 beaglebone python[2483]:  * Debug mode: on
Mar 10 03:17:52 beaglebone python[2483]: WARNING: This is a development server. Do not use it in a production deployment. Use a production WSGI server instead.
Mar 10 03:17:52 beaglebone python[2483]:  * Running on all addresses (0.0.0.0)
Mar 10 03:17:52 beaglebone python[2483]:  * Running on http://127.0.0.1:5000
Mar 10 03:17:52 beaglebone python[2483]: Press CTRL+C to quit
Mar 10 03:17:52 beaglebone python[2483]:  * Restarting with watchdog (inotify)
```

### Accessing the dashboard

Once the dashboard is running, you can access it on port 5000 on the BeagleBone. On most networks, you can use
the `beaglebone.local` hostname, which is automatically registered via multicast DNS.

> [http://beaglebone.local:5000](http://beaglebone.local:5000)

If `beaglebone.local` does not resolve, even after waiting for a few minutes following a restart, you will need to
use the IP address of the BeagleBone instead. You can find this in the output of `ifconfig`.

When it is working properly and connected to the Hestia board via I2C, the dashboard should show:

* Live "core temperature" reading, based on the U7 sensor
* Controls for turning on the heater
* Live graph showing all enabled temperature sensors that are reporting valid values
* Board image showing temperatures at different locations on the board
* Download links for all the CSV files in the `HESTIA_LOG_PATH`
* Links to the REST APIs that drive the web interface.

### Troubleshooting

If the services do not work, and you can't get any useful logging for some reason, you can try running the Python
code in your terminal.

```sh
$ cd ~/src/mawson/hestia/hestia-dash
$ source venv/bin/activate
(venv) $ cd src
(venv) $ python3 -m hestia.app
 * Serving Flask app 'hestia'
 * Debug mode: on
...
```

Or try running it interactively in Python:

```sh
$ cd ~/src/mawson/hestia/hestia-dash
$ source venv/bin/activate
(venv) $ cd src
(venv) $ python3
Python 3.7.3 (default, Oct 31 2022, 14:04:00) 
[GCC 8.3.0] on linux
Type "help", "copyright", "credits" or "license" for more information.
>>> from hestia import Hestia
>>> h = Hestia()
>>> h.read_center_temp()
24.3795
```

To debug problems with the I2C interface, you can use command line tools `i2cdetect`, `i2cdump` and `i2cget` to
try reading sensors. Here's an example reading the U7 MAX31725 sensor over I2C:

```sh
$ i2cget 2 0x4b 0x00 w
0xf319
```

In this case, we're reading from U7's configured I2C address `0x4b`. Its temperature value is in register `0x00`,
and we're reading two bytes (`w`).

The value returned is big-endian, so we need to flip it to be `0x19f3`, which is 6643 in decimal, then multiply
by the MAX31725 temperature conversion factor, which is 0.00390625. That gives us a temperature of 25.9°C.

If this command does not return a value, then check the cable connection to the Hestia board.
