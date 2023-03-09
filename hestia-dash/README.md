# hestia-dash

This is the dashboard webapp for Hestia, written in Python.

### BeagleBone Black setup

The dashboard is designed to run on a BeagleBone Black (BBB) payload computer.

To set up a new BBB from scratch, you need to:

* Connect the BBB via USB to your computer, log in via SSH using `debian`/`temppwd`

* Configure internet sharing so the BBB can connect to the internet via the USB connection:
  * For Mac, enable Internet Sharing in the Sharing preference pane, then run `/opt/scripts/network/usb_mac_ics.sh` on the BBB
  * For Linux, you will need to configure NAT on the USB interface, then can run a similar script `/opt/scripts/network/`

* Configure the TP-Link TL-WN725N Wi-Fi dongle:

```sh
$ sudo apt-get install git make gcc build-essential linux-headers-4.19.94-ti-r42
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

(For the TP-Link AC600 Wi-Fi dongle - the one with the antenna - install the [rtl8812au](https://github.com/aircrack-ng/rtl8812au.git) driver instead.)

* Install OS dependencies and check out the `hestia` source code from GitHub:

```sh
$ sudo apt-get install git python-venv
$ mkdir -p ~/src && cd ~/src
$ git clone https://github.com/mawson-rovers/hestia
```

* Create a Python virtual environment and install the dependencies:

```
$ cd ~/src/hestia/software/hestia-i2c
$ python3 -m venv venv
$ source venv/bin/activate
(venv) $ pip install -r requirements.txt
```

* You can now run the tests for the payload control software:

```
(venv) $ pytest
```

You should see some testing output and results, depending on whether the board is connected and all the sensors are working.

