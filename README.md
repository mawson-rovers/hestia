# Hestia

Hestia is a cubesat board developed by Mawson Rovers in support of UTS Resilient
Space Computing initiative. It is designed to provide temperature control and
monitoring for a demonstration heatsink payload.

Hestia was the Greek goddess of home and hearth, and similar to that, our Hestia
board provides a home and warmth to the UTS heatsink.

### Project structure

This project has three primary modules:

* **hestia-pcb** - the circuit board design for the cubesat, as a KiCAD 6 project
* **hestia-fw** - the firmware for the MSP430 microcontroller on the Hestia board,
    responsible for controlling the heater and measuring temperature, written in C
* **hestia-dash** - a web dashboard for monitoring and operating the board, used
    primarily for testing and written in Python

More details about each module can be found in their respective READMEs.

The project also has a **scripts/** folder which is used for automated builds on
GitHub, and **ws-1/** folder for interface information from our host satellite.

